const fs = require('fs').promises;
const path = require('path');

(async () => {
    let base_path = path.join(__dirname, 'raw_imgs');
    console.log(base_path);
    let files = await fs.readdir(base_path, 'utf-8');
    for (let file of files) {
        let stripped = file.replace(/[0-9]{2,4}x[0-9]{2,4}.png$/, '.png');
        try {
            await fs.access(path.join('public', 'amiibo', 'acnh', `${villager.name}.bin`));
        } catch (e) {
            let full_path = path.join(base_path, file);
            // let dest = path.join(base_path, file.replace(/\[[a-zA-Z]+] ?/, ''));
            console.log(full_path);
            // await fs.rename(full_path, dest);
        }
    }
    return 'success';
})().then(console.log).catch(e => {
    console.error('failed: ', e);
    process.exit(1);
});