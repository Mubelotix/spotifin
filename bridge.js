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

        socket.onmessage = (event) => {
            log("message:", event.data);
        };

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
