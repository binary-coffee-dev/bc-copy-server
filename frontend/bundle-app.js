const path = require('path');
const fs = require('fs');

console.log('--------------')
console.log('----BUNDLE----')
console.log('--------------')

let html = fs.readFileSync(path.join(__dirname, 'dist/index.html'), {encoding: 'utf8'});

// merge js minify into html file
{
    const js = fs.readFileSync(path.join(__dirname, 'dist/main.js'), {encoding: 'utf8'});

    // search section in string
    // <script defer="defer" src="main.js"></script>
    const p = html.indexOf("main.js");
    let s, e;
    for (let i = p; i > 0; i--) {
        if (html.charAt(i) === '<') {
            s = i;
            break;
        }
    }
    for (let i = p, j = 0; i < html.length; i++) {
        if (html.charAt(i) === '>') {
            j++;
        }
        if (j === 2) {
            e = i;
            break;
        }
    }

    // remove section and add css
    html = html.slice(0, s) + '<script>' + js + '</script>' + html.slice(e + 1);
}

// merge css minify into html file
{
    const css = fs.readFileSync(path.join(__dirname, 'dist/styles.css'), {encoding: 'utf8'});

    // search section in string
    // <link rel="stylesheet" href="styles.css">
    const p = html.indexOf("styles.css");
    let s, e;
    for (let i = p; i > 0; i--) {
        if (html.charAt(i) === '<') {
            s = i;
            break;
        }
    }
    for (let i = p; i < html.length; i++) {
        if (html.charAt(i) === '>') {
            e = i;
            break;
        }
    }

    // remove section and add css
    html = html.slice(0, s) + '<style>' + css + '</style>' + html.slice(e + 1);
}

const bundleHtml = process.argv[2] || path.join(__dirname, 'dist/bundle.html');
console.log(bundleHtml);
if (fs.existsSync(bundleHtml)) {
    fs.unlinkSync(bundleHtml);
}
fs.writeFileSync(bundleHtml, html, {flag: "wx"});
