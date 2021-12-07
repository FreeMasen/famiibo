const { generate_page: ac } = require('./js/gen_animal_crossing');
const { generate_page: ac2 } = require('./js/gen_ac_misc');
const { generate_page: index } = require('./js/gen_index');
const { generate_page: mario } = require('./js/gen_mario');
const { generate_page: zelda } = require('./js/gen_zelda');
const { generate_page: fire_emblem } = require('./js/gen_fe');
const { generate_page: metroid } = require('./js/gen_metroid');

const fns = {
    index,
    ac,
    ac2,
    mario,
    zelda,
    fire_emblem,
    metroid,
};

async function main() {
    for (let key in fns) {
        console.log('generating', key);
        let fn = fns[key];
        fn('public')
        console.log('generated', key);
    }
    return 'Complete';
}

main()
    .then(console.log)
    .catch(e => console.error('err: ', e));