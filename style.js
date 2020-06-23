require('purecss')
import './static/commits-loupe-style.css'

function classRemap(mapping) {
    for (var selector in mapping) {
        let addClasses = mapping[selector];
        document.querySelectorAll(selector).forEach((e) => {
            e.classList.add(...addClasses);
        });
    }
}

function remapLoupeCss() {
    console.debug("Remap Loupe CSS");
    classRemap({
        ".loupe-panels": ["pure-g"],
        ".loupe-container": ["pure-u-1-3"],
        ".loupe-button": ["pure-button"],
        ".loupe-commits-table": ["pure-table", "pure-table-striped"]
    });
}

export function initStyle() {
    const observer = new MutationObserver((mList, ob) => { remapLoupeCss(); });
    document.querySelectorAll(".loupe-root").forEach((elem) => {
        observer.observe(elem, {
            attributes: true,
            childList: true,
            subTree: true
        });
    });
    remapLoupeCss();
}
