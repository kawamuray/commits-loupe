export { ready };

const publicPath = document.currentScript.src;

function ready(cb) {
    // Set webpack public path dynamically to load extra resources from the same
    // location to this script.
    // Say this script is loaded from `https://xx.com/js/xx.js`,
    // we set __webpack_public_path__ to `https://xx.com/js/` so that
    // other script yy.js loaded from `https://xx.com/js/yy.js`.
    var url = new URL(publicPath);
    var items = url.pathname.split('/');
    items[items.length - 1] = '';
    url.pathname = items.join('/');
    console.log("public path = " + url.toString());
    const path_save = __webpack_public_path__;
    __webpack_public_path__ = url.toString();

    import('./pkg').then(cb);

    __webpack_public_path__ = path_save;
}
