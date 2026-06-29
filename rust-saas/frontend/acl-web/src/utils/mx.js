var mxBasePath = '/mxgraph';
var mxLoadResources = true;
var mxResourceExtension = '.txt';

async function loadmodule() {
    let _= await loadScript("/mxgraph/js/Init.js");
     _= await loadScript("/mxgraph/deflate/base64.js");
     _= await loadScript("/mxgraph/deflate/pako.min.js");
     _= await loadScript("/mxgraph/jscolor/jscolor.js");
     _= await loadScript("/mxgraph/sanitizer/sanitizer.min.js");
     _= await loadScript("/mxgraph/js/mxClient.js");
     console.log('loadmodule')
    return new Promise(resolve => resolve());
}

async function loadScript(scriptUrl) {
    const script = document.createElement('script');
    script.src = scriptUrl;
    document.head.appendChild(script);
    await new Promise(resolve => script.onload = resolve);
}

export { loadmodule };