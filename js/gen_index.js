const fs = require('fs').promises;
const path = require('path');
const prefix = require('./prefix');

const NAMES = Object.freeze({
    acnh:  'Animal Crossing Villagers',
    zelda: 'The Legend of Zelda',
    mario: 'Super Mario Brothers',
    'fire-emblem': 'Fire Emblem',
    'ac-misc': 'Animal Crossing',
    metroid: 'Metroid',
});

let html = `${prefix.html(['style/index.css'])}
        <ul id="amiibo-set-list">
`;

async function gen_index(base) {
    const files = await fs.readdir(path.join('public', 'amiibo'));
    for (const dir of files) {
        html += li_for_dir(dir);
    }
    html += `
    </ul>
    <script type="text/javascript" src="shared.js"></script>
    </body></html>`;
    await fs.writeFile(path.join(base, 'index.html'), html);
}

function li_for_dir(name) {
    return `<li class="amiibo-set">
        <a class="amiibo-link" href="/${name}">
            <img class="amiibo-image" src="images/${name}.png" />
            <span class="game-title">${NAMES[name]}</span>
        </a>
    </li>
`;
}

module.exports.generate_page = gen_index;