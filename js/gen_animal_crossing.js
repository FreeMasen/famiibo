const fs = require('fs').promises;
const path = require('path');
const prefix = require('./prefix.js');

let html = `${prefix.html(['style/acnh.css'])}
        <table>
            <thead>
                <tr>
                    <th align="center" class="cell-name">Name</th>
                    <th align="center" class="cell-gender">Gender</th>
                    <th align="center" class="cell-personality">Personality</th>
                    <th align="center" class="cell-species">Species</th>
                    <th align="center" class="cell-birthday">Birthday</th>
                </tr>
                <tr>
                    <th align="center" class="cell-name">
                        <input id="search-name" type="text"></input>
                    </th>
                    <th align="center" class="cell-gender">
                        <select id="search-gender">
                            <option value=""></option>
                            <option value="Male">Male</option>
                            <option value="Female">Female</option>
                        </select>
                    </th>
                    <th align="center" class="cell-personality">
                        <select id="search-personality">
                            <option value=""></option>
                            <option id="cranky" value="Cranky">Cranky</option>
                            <option id="jock" value="Jock">Jock</option>
                            <option id="lazy" value="Lazy">Lazy</option>
                            <option id="normal" value="Normal">Normal</option>
                            <option id="peppy" value="Peppy">Peppy</option>
                            <option id="sisterly" value="Sisterly">Sisterly</option>
                            <option id="smug" value="Smug">Smug</option>
                            <option id="snooty" value="Snooty">Snooty</option>
                        </select>
                    </th>
                    <th align="center" class="cell-species">
                        <select id="search-species">
                            <option value=""></option>
                            <option value="Alligator">Alligator</option>
                            <option value="Anteater">Anteater</option>
                            <option value="Bear">Bear</option>
                            <option value="Bird">Bird</option>
                            <option value="Bull">Bull</option>
                            <option value="Cat">Cat</option>
                            <option value="Chicken">Chicken</option>
                            <option value="Cow">Cow</option>
                            <option value="Cub">Cub</option>
                            <option value="Deer">Deer</option>
                            <option value="Dog">Dog</option>
                            <option value="Duck">Duck</option>
                            <option value="Eagle">Eagle</option>
                            <option value="Elephant">Elephant</option>
                            <option value="Frog">Frog</option>
                            <option value="Goat">Goat</option>
                            <option value="Gorilla">Gorilla</option>
                            <option value="Hamster">Hamster</option>
                            <option value="Hippo">Hippo</option>
                            <option value="Horse">Horse</option>
                            <option value="Kangaroo">Kangaroo</option>
                            <option value="Koala">Koala</option>
                            <option value="Lion">Lion</option>
                            <option value="Monkey">Monkey</option>
                            <option value="Mouse">Mouse</option>
                            <option value="Octopus">Octopus</option>
                            <option value="Ostrich">Ostrich</option>
                            <option value="Penguin">Penguin</option>
                            <option value="Pig">Pig</option>
                            <option value="Rabbit">Rabbit</option>
                            <option value="Rhino">Rhino</option>
                            <option value="Sheep">Sheep</option>
                            <option value="Squirrel">Squirrel</option>
                            <option value="Tiger">Tiger</option>
                            <option value="Wolf">Wolf</option>
                        </select>
                    </th>
                    <th align="center" class="cell-birthday">
                        <select id="search-month">
                            <option value=""></option>
                            <option value="January">Jan</option>
                            <option value="February">Feb</option>
                            <option value="March">Mar</option>
                            <option value="April">Apr</option>
                            <option value="May">May</option>
                            <option value="June">Jun</option>
                            <option value="July">Jul</option>
                            <option value="August">Aug</option>
                            <option value="September">Sep</option>
                            <option value="October">Oct</option>
                            <option value="November">Nov</option>
                            <option value="December">Dec</option>
                        </select>
                        <select id="search-day">
                            <option value=""></option>
                            <option value="1">1st</option>
                            <option value="2">2nd</option>
                            <option value="3">3rd</option>
                            <option value="4">4th</option>
                            <option value="5">5th</option>
                            <option value="6">6th</option>
                            <option value="7">7th</option>
                            <option value="8">8th</option>
                            <option value="9">9th</option>
                            <option value="10">10th</option>
                            <option value="11">11th</option>
                            <option value="12">12th</option>
                            <option value="13">13th</option>
                            <option value="14">14th</option>
                            <option value="15">15th</option>
                            <option value="16">16th</option>
                            <option value="17">17th</option>
                            <option value="18">18th</option>
                            <option value="19">19th</option>
                            <option value="20">20th</option>
                            <option value="21">21st</option>
                            <option value="22">22nd</option>
                            <option value="23">23rd</option>
                            <option value="24">24th</option>
                            <option value="25">25th</option>
                            <option value="26">26th</option>
                            <option value="27">27th</option>
                            <option value="28">28th</option>
                            <option value="29">29th</option>
                            <option value="30">30th</option>
                            <option value="31">31st</option>
                        </select>
                    </th>
                </tr>
            </thead>
            <tbody>
    `;

async function generate_page(base_path) {
    const json = await fs.readFile(path.join(__dirname, 'villager_list.json'), 'utf-8');
    const villagers = JSON.parse(json);
    for (const villager of villagers) {
        html += row_for_villager(villager)
    }
    html += `
            </tbody>
        </table>
        <script type="text/javascript" src="shared.js"></script>
        <script type="text/javascript" src="acnh.js"></script>
    </body>
</html>`;
    const dir = path.join(base_path, 'acnh');
    if (!(await fs.access(dir).then(() => true).catch(() => false))) {
        await fs.mkdir(dir);
    }
    await fs.writeFile(path.join(dir, 'index.html'), html);
}

async function row_for_villager(villager) {
    let gender = villager.gender === '♂' ? 'Male' : 'Female';
    const { month, day } = get_birthday(villager.birthday);
    let btn_class = 'write-button';
    try {
        await fs.access(path.join('public', 'amiibo', 'acnh', `${villager.name}.bin`));
    } catch (e) {
        console.error('cannot access amiibo bin file', e);
        btn_class = ' disabled';
    }
    return `<tr
        class="villager-row"
        data-name="${villager.name}"
        data-gender="${gender}"
        data-personality="${villager.personality}"
        data-species="${villager.species}"
        data-month="${month}"
        data-day="${day}">
    <td class="cell-name">
        <div>
            <img src="${villager.image_url}" class="villager-picture" />
            <span class="villager-name">${villager.name}</span>
            <button data-url="write/acnh/${villager.name}" class="${btn_class}">Write</button>
        </div>
    </td>
    <td align="center" class="cell-gender">${gender}</td>
    <td align="center" class="cell-personality">${villager.personality}</td>
    <td align="center" class="cell-species">${villager.species}</td>
    <td align="center" class="cell-birthday">${villager.birthday}</td>
</tr>`;
}


function get_birthday(dt) {
    let parts = dt.split(' ');
    return {
        month: parts[0],
        day: parts[1].replace(/th|nd|rd|st/, '')
    }
}


module.exports.generate_page = generate_page;
