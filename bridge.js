// bridge.js — connects the Spotify client to the backend WebSocket.
(function bridge() {
    const WS_URL = "ws://127.0.0.1:8000/ws";
    const RETRY_MS = 2000;
    let socket = null;
    let retryTimer = null;

    function log(...args) {
        console.log("[bridge]", ...args);
    }

    function scheduleReconnect() {
        if (retryTimer !== null) return;
        retryTimer = setTimeout(() => {
            retryTimer = null;
            connect();
        }, RETRY_MS);
    }

    function serialize(value) {
        if (value === undefined) return "undefined";
        if (typeof value === "string") return value;
        try {
            return JSON.stringify(value);
        } catch (error) {
            return String(value);
        }
    }

    function handleMessage(event) {
        let message;
        try {
            message = JSON.parse(event.data);
        } catch (error) {
            log("bad message:", error);
            return;
        }
        if (message.type !== "eval") {
            log("message:", event.data);
            return;
        }

        const respond = (response) => {
            if (!socket || socket.readyState !== WebSocket.OPEN) {
                log("cannot respond, socket not open");
                return;
            }
            try {
                socket.send(JSON.stringify(response));
            } catch (error) {
                log("send failed:", error);
            }
        };

        try {
            Promise.resolve(eval(message.code)).then(
                (value) => respond({ type: "result", id: message.id, ok: true, value: serialize(value) }),
                (error) => respond({ type: "result", id: message.id, ok: false, error: String((error && error.message) || error) })
            );
        } catch (error) {
            respond({ type: "result", id: message.id, ok: false, error: String((error && error.message) || error) });
        }
    }

    function connect() {
        if (socket && socket.readyState <= WebSocket.OPEN) return;
        try {
            socket = new WebSocket(WS_URL);
        } catch (error) {
            log("connect error:", error);
            socket = null;
            scheduleReconnect();
            return;
        }

        socket.onopen = () => {
            log("connected to", WS_URL);
            socket.send(JSON.stringify({ type: "hello", agent: "spicetify-bridge" }));
        };

        socket.onmessage = handleMessage;

        socket.onerror = () => {
            log("error");
        };

        socket.onclose = (event) => {
            log("closed (code " + event.code + "), reconnecting in " + RETRY_MS + "ms");
            socket = null;
            scheduleReconnect();
        };
    }

    connect();
    log("started, target " + WS_URL);
})();
