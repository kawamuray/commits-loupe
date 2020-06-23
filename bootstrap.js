export { ready };

function ready(cb) {
    import('./pkg').then(cb);
}
