export const parseMarkdown = (text: string) => {
  let result = text
    .replace(/\*\*(.+?)\*\*/g, '<strong>$1</strong>')
    .replace(/\[DONE\]/g, '.')
  
  const lines = result.split('\n')
  const processedLines: string[] = []
  let inList = false
  
  for (const line of lines) {
    const headingMatch = line.match(/^(#{1,6})\s+(.+)$/)
    if (headingMatch) {
      if (inList) {
        processedLines.push('</div>')
        inList = false
      }
      const level = headingMatch[1].length
      processedLines.push(`<h${level} class="heading-${level}">${headingMatch[2].trim()}</h${level}>`)
    } else if (/^---+$/.test(line)) {
      if (inList) {
        processedLines.push('</div>')
        inList = false
      }
      processedLines.push('<hr class="divider" />')
    } else {
      const listMatch = line.match(/^\*\s+(.+)$/)
      if (listMatch) {
        if (!inList) {
          processedLines.push('<div class="list-container">')
          inList = true
        }
        processedLines.push(`<div class="list-item" style="display:block;">➢ ${listMatch[1].trim()}</div>`)
      } else {
        if (inList) {
          processedLines.push('</div>')
          inList = false
        }
        processedLines.push(line.replace(/\*(.+?)\*/g, '<em>$1</em>'))
      }
    }
  }
  
  if (inList) {
    processedLines.push('</div>')
  }
  

  let start = false,head = false;
  for(let i = 0; i < processedLines.length; i++) {
    let line = processedLines[i]
    let match =  /([^\|]+?)\|/g;
    let matches =[... line.matchAll(match)];
    if(matches.length === 0) {
      if(start) {
        processedLines[i] = '</table>'+ processedLines[i]
      }
      start = false
      continue
    } 

    if(!start) {
      start = true
      head = true
    }

    let list = []
    for(let m of matches) {
      list.push(m[1].replace(':---', '').trim())
    }
   
    let flag = false;
    for(let v of list) {
      if(v !== '') {
        flag = true
        break
      }
    }

    if(!flag) {
      processedLines[i] = ""
      continue
    }

    console.log('list', list)

    let body='';
    if(head) {
      head = false
      body =  '<table class="flex-1 w-full wrap text-center table table-striped table-hover table-bordered table-sm table-responsive-md ">'
            + '<tr><td class="text-center font-bold border border-surface-900 px-2 py-2 bg-gray-600 text-white">' 
            + list.join('</td><td class="text-center font-bold border border-surface-900 px-2 py-2 bg-gray-600 text-white">') + '</td></tr>'
    }
    else {
      body = '<tr><td class="text-left border border-surface-900 px-2 py-2 bg-white">' 
      + list.join('</td><td class="text-left border border-surface-900 px-2 py-2 bg-white">') + '</td></tr>'
    }
    
    processedLines[i] = body
  }

  return processedLines.join('\n')
}
