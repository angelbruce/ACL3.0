var mxBasePath = '/mxgraph';
var mxLoadResources = true;
var mxResourceExtension = '.txt';

async function loadmodule() {
    // await loadScript("/mxgraph/js/Init.js");
    // await loadScript("/mxgraph/deflate/base64.js");
    // await loadScript("/mxgraph/deflate/pako.min.js");
    // await loadScript("/mxgraph/jscolor/jscolor.js");
    // await loadScript("/mxgraph/sanitizer/sanitizer.min.js");
    // await loadScript("/mxgraph/js/mxClient.js");

         loadScript("/mxgraph/js/Init.js").then(() => {
            loadScript("/mxgraph/deflate/base64.js").then(() => {
                loadScript("/mxgraph/deflate/pako.min.js").then(() => {
                    loadScript("/mxgraph/jscolor/jscolor.js").then(() => {
                        loadScript("/mxgraph/sanitizer/sanitizer.min.js").then(() => {
                            loadScript("/mxgraph/js/mxClient.js").then(() => {
                                console.log("mxClient.js loaded");
                            });
                        });
                    });
                });
            });
        });
}

async function loadScript(scriptUrl) {
    const script = document.createElement('script');
    script.src = scriptUrl;
    document.head.appendChild(script);
    await new Promise(resolve => script.onload = resolve);
}

export { loadmodule };