const fs = require('fs').promises;
const path = require('path');
const prefix = require('./prefix.js');

let html = `${prefix.html(['style/mario.css'])}
    <ul id="amiibo-list">`;

module.exports.generate_page = async function (base) {
    const files = await fs.readdir(path.join(base, 'amiibo', 'zelda'));
    for (const file of files) {
        html += li_for_bin(file);
    }
    html += `
        </ul>
        <script src="shared.js" type="text/javascript"></script>
        <script src="zelda.js" type="text/javascript"></script>
    </body>    
</html>`;
    await fs.writeFile(path.join(base, 'zelda', 'index.html'), html);
}


function li_for_bin(bin) {
    let name = bin.replace('.bin', '');
    let img_name = name.replace('[Special Data]', '').trim();
    return `
<li class="amiibo-list-entry">
    <img class="amiibo-picture" src="images/zelda/${img_name}.png" />
    <span class="amiibo-name">${name}</span>
    <button data-url="write/zelda/${bin}" class="write-button">Write</button>
</li>`
}