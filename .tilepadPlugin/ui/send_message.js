

const messageIn = document.getElementById("message");

messageIn.onchange = (event) => {
    const value = event.target.value;
    tilepad.tile.setProperty("message", value);
}

messageIn.setAttribute('disabled', '')

tilepad.tile.getProperties()
    .then((properties) => {
        messageIn.value = (properties.message ?? '');
        messageIn.removeAttribute('disabled');
    })