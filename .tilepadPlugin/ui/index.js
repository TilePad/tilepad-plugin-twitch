// === Screens ===

const screenIds = [
    "connectingScreen",
    "authorizeScreen",
    "noActionsScreen",
    "sendMessageScreen"
];

function setActiveScreen(id) {
    // Hide existing screens
    for (const screenId of screenIds) {
        const screenEl = document.getElementById(screenId);
        screenEl.classList.remove('screen--visible')
    }

    // Show current screen
    const screenEl = document.getElementById(id);
    screenEl.classList.add('screen--visible')
}

// === Authorize Screen ===

const authorizeBtn = document.getElementById("authorize");

authorizeBtn.onclick = () => {
    tilepad.plugin.send({
        type: "OPEN_AUTH_URL"
    });

    authorizeBtn.innerText = "Authorizing..."
    authorizeBtn.setAttribute('disabled', '');
}

// === Send Message Screen ===

const messageIn = document.getElementById("message");

messageIn.onchange = (event) => {
    const value = event.target.value;
    tilepad.tile.setProperty("message", value);
}

messageIn.setAttribute('disabled', '')

tilepad.tile.onProperties((properties) => {
    if (currentAction !== "send_message") return;

    messageIn.value = (properties.message ?? '');
    messageIn.removeAttribute('disabled');
})


// === Current State ===

let currentAction = null;

tilepad.plugin.onMessage((data) => {
    switch (data.type) {
        case "STATE": {
            const state = data.state;
            switch (state) {
                case "LOADING": {
                    setActiveScreen("connectingScreen")
                    break;
                }
                case "NOT_AUTHENTICATED": {
                    setActiveScreen("authorizeScreen")
                    break;
                }
                case "AUTHENTICATED": {
                    switch (currentAction) {
                        case "send_message": {
                            setActiveScreen("sendMessageScreen")
                            break;
                        }

                        // Default page for no additional options
                        default: {
                            setActiveScreen("noActionsScreen")
                            break;
                        }
                    }

                    break;
                }
            }

            break;
        }
    }
});


// Request the current properties
tilepad.tile.requestProperties();

// Request the current tile
tilepad.tile.getTile()
    // Handle properties received
    .then((tile) => {
        currentAction = tile.actionId;

        // Request connection state from the plugin
        tilepad.plugin.send({
            type: "GET_STATE"
        });
    });