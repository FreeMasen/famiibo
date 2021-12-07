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
    function handle_message(msg) {
        let info
        try {
            info = JSON.parse(msg);
        } catch (e) {
            return console.error('error parsing json', msg, e);
        }
        switch (info.kind) {
            case "success": {
                set_message("Complete!", false);
                es.close();
                break;
            }
            case "error": {
                set_message(`Error: ${info.data}`, true);
                es.close();
                break;
            }
            case "stdOut": {
                set_message(info.data, false);
                break;
            }
            case "stdErr": {
                set_message(info.data, true);
                break;
            }
            case "ping": {
                set_message("...", false);
            }
            default: {
                set_message(`${info.kind}: ${info.data | ''}`);
            }
        }
    }
    function write_amiibo2(ev) {
        let es = new EventSource(ev.currentTarget.dataset.url.replace(/\/?write\//, '/exe/'));
        es.onerror = function(ev) {
            set_message('Error on connection');
            es.close();
            console.error(ev);
        }
        es.onmessage = function(ev) {
            handle_message(ev.data);
        }
        es.onopen = function() {
            set_message('starting write!');
        }
        es.addEventListener('complete', function() {
            console.log('closing...');
            es.close();
        });
        es.addEventListener('timeout', function() {
            set_message('timedout', true);
            es.close();
        });
    }
    window.set_message = set_message;
    window.write_amiibo = write_amiibo2;
})();