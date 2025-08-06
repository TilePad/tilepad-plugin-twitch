const authorizeBtn = document.getElementById("authorize");

authorizeBtn.onclick = () => {
    tilepad.plugin.send({
        type: "OPEN_AUTH_URL"
    });

    authorizeBtn.innerText = "Authorizing..."
    authorizeBtn.setAttribute('disabled', '');
}

