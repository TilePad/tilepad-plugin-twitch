const containerEl = document.getElementById("container")
const counterEl = document.getElementById("counter")

tilepad.plugin.onMessage((message) => {
    switch (message.type) {
        case "VIEW_COUNT": {
            counterEl.innerText = message.count;
            fitTextToContainer(counterEl, containerEl);
            break;
        }
    }
})


function updateViewCount() {
    tilepad.plugin.send({ type: "GET_VIEW_COUNT" })
}

function fitTextToContainer(element, container) {
    const paddingX = container.clientWidth * 0.1;
    const paddingY = container.clientWidth * 0.1;

    let fontSize = 100;
    element.style.fontSize = fontSize + "px";

    while (
        (element.scrollWidth > container.clientWidth - (paddingX * 2) ||
            element.scrollHeight > container.clientHeight - (paddingY * 2)) &&
        fontSize > 0
    ) {
        fontSize--;
        element.style.fontSize = fontSize + "px";
    }
}

window.addEventListener("resize", () => fitTextToContainer(counterEl, containerEl));

updateViewCount();

setInterval(() => {
    updateViewCount();
}, 2000);