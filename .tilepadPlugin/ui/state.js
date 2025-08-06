let currentAction = null;

tilepad.plugin.onMessage((data) => {
    switch (data.type) {
        case "STATE": {
            const state = data.state;
            switch (state) {
                case "NOT_AUTHENTICATED": {
                    if (!window.location.pathname.endsWith('ui/connect.html')) {
                        window.location.href = "./connect.html";
                    }
                    break;
                }
                case "LOADING": {
                    if (!window.location.pathname.endsWith('ui/connecting.html')) {
                        window.location.href = "./connecting.html";
                    }
                    break;
                }
                case "AUTHENTICATED": {
                    switch (currentAction) {
                        case "send_message": {
                            if (!window.location.pathname.endsWith('ui/send_message.html')) {
                                window.location.href = "./send_message.html";
                            }
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