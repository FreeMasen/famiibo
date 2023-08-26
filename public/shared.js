(function() {
    const MESSAGE = document.getElementById('message');
    let TO
    function set_message(msg, err) {
        if (!msg) {
            return;
        }
        console.log(msg, err);
        MESSAGE.innerText = msg;
        if (TO) {
            clearTimeout(TO);
        }
        if (err) {
            MESSAGE.classList.add('error');
            MESSAGE.classList.remove('info');
        } else {
            MESSAGE.classList.add('info');
            MESSAGE.classList.remove('error');
        }
        TO = setTimeout(clear_message, 8 * 1000);
    }
    function clear_message() {
        MESSAGE.classList.remove('error');
        MESSAGE.classList.remove('info');
        MESSAGE.innerText = '';
    }
    function write_amiibo(ev) {
        /**
         * @type {EventSource}
         */
        let sse
        try {
            sse = new EventSource(ev.currentTarget.dataset.url);
        } catch (e) {
            return set_message(e.message, true)
        }
        sse.onopen = (e) => {
            console.log("Opened!", e);
        };
        sse.onmessage = (e) => {
            let info = JSON.parse(e.data);
            console.log("MSG: ", e.data);
            switch (info.type) {
                case "started": {
                    set_message("started")
                    break;
                }
                case "progress": {
                    set_message(`${info.data.toFixed(2)}% complete`)
                    break;
                }
                case "success": {
                    set_message("Complete!");
                    sse.close();
                    break;
                }
                case "failed": {
                    set_message(info.data, true);
                    sse.close();
                }
            }
        };
        sse.onclose = (ev) => {
            console.error("close:", ev)
            return false;
        };
        sse.onerror = ev => {
            console.error("error: ", ev);
        }
    }
    window.set_message = set_message;
    window.write_amiibo = write_amiibo;
    let buttons = Array.from(document.querySelectorAll(".write-button"));
    for (let b of buttons) {
        b.addEventListener("click", write_amiibo);
    }
})();
