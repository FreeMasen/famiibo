const fs = require('fs').promises;
const cp = require('child_process');
const path = require('path');

async function main() {
    let text = await fs.readFile('amiibo.txt', 'utf-8');
    for (const url of text.split('\n')) {
        let parts = url.split('/');
        let name = `${parts[parts.length - 1]}.png`;
        if (name == 'figure.png') {
            name = `${parts[parts.length - 2]}.png`;
        }
        await execute('curl', [`https://www.nintendo.com${url}`, '-o', path.join('amiibo_images', name)])
    }
    return 'Success!';
}

async function execute(cmd, args) {
    return new Promise((resolve, reject) => {
        cp.exec(`${cmd} ${args.join(' ')}`, (e, out, err) => {
            if (e) {
                console.error(e);
                console.error(err);
                return reject(e);
            }
            console.log(out);
            resolve();
        });
    });
}

main().then(console.log).catch(console.error);