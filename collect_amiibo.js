let fs = require("fs").promises;
let path = require("path");
let prefix = /(\[AC] )?[A-Z0-9]{3}\s*-\s*/

async function search(dir, name) {
    let files = await fs.readdir(dir);
    for (let file of files) {
        let full_path = path.join(dir, file);
        let meta = await fs.lstat(full_path);
        if (meta.isDirectory()) {
            let result = await search(path.join(dir, file), name);
            if (!!result) return result;
        } else {
            let fname = file.trim().replace(prefix, "");
            if (fname.toLocaleLowerCase().startsWith(name.toLocaleLowerCase())) {
                return full_path;
            }
        }
    }
}

(async () => {
    const json = await fs.readFile(path.join(__dirname, "js", "villager_list.json"), 'utf-8');
    let list = JSON.parse(json);
    let dir = path.join(__dirname, "Amiibo Bins", "!Animal Crossing Amiibo");
    for (let v of list) {
        let name = v.name.toLocaleLowerCase().replace('é', 'e').replace("'", "_");
        let bin_path = await search(dir, name);
        if (bin_path) {
            await fs.copyFile(bin_path, path.join(__dirname, "public", "amiibo", "acnh2", `${v.name}.bin`));
            // console.log(bin_path, path.join(__dirname, "public", "amiibo", "acnh", `${v.name}.bin`));
        } else {
            console.log('no bin for', v.name);
        }
    }
    return 'Ok!'
})().then(console.log).catch(console.error);
