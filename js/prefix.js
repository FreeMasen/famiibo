module.exports.html = function (css_list, title) {
    return `<!DOCTYPE html>
    <html>
        <head>
            <meta name="viewport" content="width=device-width, initial-scale=1" />
            <meta charset="utf-8" />
            <link rel="stylesheet" href="style.css" type="text/css" />
            ${css_list.map(name => `<link rel="stylesheet" text="text/css" href="${name}" />`).join('\n')}
            <title>Famiibo${title  ? ' - ' + title : ''}</title>
        </head>
        <body>
            <div id="message">
            </div>
`;
}