(function() {
    const SEARCH_ELEMENTS = Object.freeze({
        /** @type {HTMLInputElement} */
        NAME: document.getElementById('search-name'),
        /** @type {HTMLSelectElement} */
        GENDER: document.getElementById('search-gender'),
        /** @type {HTMLSelectElement} */
        PERSONALITY: document.getElementById('search-personality'),
        /** @type {HTMLSelectElement} */
        SPECIES: document.getElementById('search-species'),
        /** @type {HTMLSelectElement} */
        MONTH: document.getElementById('search-month'),
        /** @type {HTMLSelectElement} */
        DAY: document.getElementById('search-day'),
    });
    const CURRENT_SEARCH = {
        name: null,
        gender: null,
        personality: null,
        species: null,
        month: null,
        day: null,
    };
    
    /**
     * 
     * @param {Event} ev 
     */
    function search_element_change(ev) {
        update_search_values()
        update_search_elements();
        update_table();
    }

    function update_search_values() {
        update_search_value(SEARCH_ELEMENTS.NAME.value, 'name');
        update_search_value(SEARCH_ELEMENTS.GENDER.value, 'gender');
        update_search_value(SEARCH_ELEMENTS.PERSONALITY.value, 'personality');
        update_search_value(SEARCH_ELEMENTS.SPECIES.value, 'species');
        update_search_value(SEARCH_ELEMENTS.MONTH.value, 'month');
        update_search_value(SEARCH_ELEMENTS.DAY.value, 'day');
    }

    function update_search_value(el_value, key) {
        if (el_value === '') {
            CURRENT_SEARCH[key] = null;
        } else {
            CURRENT_SEARCH[key] = el_value
        }
    }

    function update_search_elements() {
        update_days();
        update_months();
        update_gender();
        update_personality();
    }

    function update_gender() {
        switch (CURRENT_SEARCH.personality) {
            case 'Cranky':
            case 'Smug':
            case 'Jock':
            case 'Lazy':
                SEARCH_ELEMENTS.GENDER.firstChild.disabled = false;
                SEARCH_ELEMENTS.GENDER.lastChild.disabled = true;
                break;
            case 'Peppy':
            case 'Sisterly':
            case 'Normal':
            case 'Snooty':
                SEARCH_ELEMENTS.GENDER.firstChild.disabled = false;
                SEARCH_ELEMENTS.GENDER.lastChild.disabled = true;
            default:
                SEARCH_ELEMENTS.GENDER.firstChild.disabled = false;
                SEARCH_ELEMENTS.GENDER.lastChild.disabled = true;
        }
    }

    function get_gender_ids(gender) {
        if (gender == 'Male') {
            return ['cranky', 'smug', 'jock', 'lazy'];
        } else {
            return [
                'peppy',
                'sisterly',
                'normal',
                'snooty',
            ];
        }
    }

    function update_personality() {
        let opposite = CURRENT_SEARCH.gender === 'Male' ? 'Female' : 'Male';
        set_personality_set(CURRENT_SEARCH.gender);
        set_personality_set(opposite);
    }

    function set_personality_set(gender, disabled) {
        for (const id of get_gender_ids(gender)) {
            const el = document.getElementById(id);
            el.disabled = disabled;
        }
    }

    function update_days() {
        switch (CURRENT_SEARCH.month) {
            case 'Jan': return set_day_disabled(31);
            case 'Feb': return set_day_disabled(28);
            case 'Mar': return set_day_disabled(31);
            case 'Apr': return set_day_disabled(30);
            case 'May': return set_day_disabled(31);
            case 'Jun': return set_day_disabled(30);
            case 'Jul': return set_day_disabled(31);
            case 'Aug': return set_day_disabled(31);
            case 'Sep': return set_day_disabled(30);
            case 'Oct': return set_day_disabled(31);
            case 'Nov': return set_day_disabled(30);
            case 'Dec': return set_day_disabled(31);
        }
    }

    function update_months() {
        for (let i = 0; i < SEARCH_ELEMENTS.MONTH.childNodes.length; i++) {
            let month = SEARCH_ELEMENTS.MONTH.childNodes[i];
            set_month_disabled(month);
        }
    }
    function set_month_disabled(month) {
        switch (month.value) {
            case 'Jan': 
            case 'Mar':
            case 'May':
            case 'Jul':
            case 'Aug':
            case 'Oct':
            case 'Dec':
                month.disabled = CURRENT_SEARCH.day < 31;
                break;
            case 'Apr':
            case 'Jun':
            case 'Sep':
            case 'Nov':
                month.disabled = CURRENT_SEARCH.day < 30;
                break;
            case 'Feb':
                month.disabled = false;
                break;
        }
    }
    function set_day_disabled(last_day) {
        SEARCH_ELEMENTS.DAY.lastElementChild.disabled = last_day < 31;
        SEARCH_ELEMENTS.DAY.lastElementChild.previousSibling.disabled = last_day < 30;
        SEARCH_ELEMENTS.DAY.lastElementChild.previousSibling.previousSibling.disabled = last_day < 29;
    }

    function update_table() {
        let rows = Array.from(document.querySelectorAll('.villager-row'));
        for (const row of rows) {
            let name = true;
            if (CURRENT_SEARCH.name === null) {
                name = true;
            } else {
                name = row.dataset.name.toLowerCase().startsWith(CURRENT_SEARCH.name);
            }
            handle_show_hide(row, {
                name,
                gender: is_match(row, 'gender'),
                personality: is_match(row, 'personality'),
                species: is_match(row, 'species'),
                month: is_match(row, 'month'),
                day: is_match(row, 'day'),
            });
        }
    }

    function is_match(el, key) {
        if (CURRENT_SEARCH[key] === null) {
            return true;
        }
        return el.dataset[key] === CURRENT_SEARCH[key];
    }
    function handle_show_hide(el, {name, gender, personality, species, month, day}) {
        if (name && gender && personality && species && month && day) {
            el.classList.remove('hidden');
        } else {
            el.classList.add('hidden');
        }
    }
    /**
     * 
     * @param {Event} ev 
     */
    async function write_clicked(ev) {
        set_all_buttons_disabled(true);
        let btn = ev.currentTarget;
        let row = btn.parentElement.parentElement.parentElement;
        let name = row.dataset.name;
        try {
            let reply = await fetch(`/write/acnh/${name}`, {
                method: 'POST',
            });
            if (reply.status === 200) {
                window.set_message(`Successfully wrote amiibo for ${name} to NFC chip`);
            } else {
                window.set_message(`Failed to write amiibo for ${name}`, true);
                let body = await reply.json();
                console.error(body);
            }
        } catch (err) {
            window.set_message(`Failed to write amiibo for ${name}`, true);
            console.error(err);
        } finally {
            set_all_buttons_disabled(false);
        }
    }
    function set_all_buttons_disabled(disabled) {
        let buttons = document.querySelectorAll('.write-button');
        for (let i = 0; i < buttons.length; i++) {
            let button = buttons[i];
            if (disabled) {
                button.setAttribute('disabled', '');
                button.classList.add('disabled');
            } else {
                button.removeAttribute('disabled');
                button.classList.remove('disabled');
            }
        }
    }
    
    (function () {
        SEARCH_ELEMENTS.NAME.addEventListener('input', search_element_change);
        SEARCH_ELEMENTS.GENDER.addEventListener('change', search_element_change);
        SEARCH_ELEMENTS.PERSONALITY.addEventListener('change', search_element_change);
        SEARCH_ELEMENTS.SPECIES.addEventListener('change', search_element_change);
        SEARCH_ELEMENTS.MONTH.addEventListener('change', search_element_change);
        SEARCH_ELEMENTS.DAY.addEventListener('change', search_element_change);
        let buttons = document.querySelectorAll('.write-button');
        for (let i = 0; i < buttons.length; i++) {
            let button = buttons[i];
            button.addEventListener('click',write_clicked);
        }
    })();

})()