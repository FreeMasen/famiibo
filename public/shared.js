(function() {
    const MESSAGE = document.getElementById('message');
    function set_message(msg, err) {
        console.log(msg, err);
        MESSAGE.innerText = msg;
        if (err) {
            MESSAGE.classList.add('error');
        } else {
            MESSAGE.classList.add('info');
        }
        setTimeout(clear_message, 8 * 1000);
    }
    function clear_message() {
        MESSAGE.classList.remove('error');
        MESSAGE.classList.remove('info');
        MESSAGE.innerText = '';
    }
    function write_amiibo(ev) {
        fetch(ev.currentTarget.dataset.url, {
            method: 'POST'
        }).then(r => {
            if (r.status === 200) {
                return r.json();
            } else {
                return r.text().then(text => {
                    console.error('error:', text);
                    throw new Error(text);
                });
            }
        })
        .then(body => {
            console.log('success', body);
            set_message('Successfully wrote amiibo', false);
        })
        .catch(e => {
            set_message('Failed to write amiibo', true);
        });
    }
    window.set_message = set_message;
    window.write_amiibo = write_amiibo;
})();