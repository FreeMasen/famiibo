(function() {
    const buttons = document.querySelectorAll('.write-button');
    for (let i = 0; i < buttons.length; i++) {
        let btn = buttons[i];
        btn.addEventListener('click', write_amiibo);
    }
})();