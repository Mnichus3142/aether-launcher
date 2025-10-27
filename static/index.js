const inputField = document.getElementById("userInput");

const wsUri = "ws://127.0.0.1:3000";
const websocket = new WebSocket(wsUri);

let actualIndex = 0;
let maxIndex = 0;
let app = "";
let previousIndex = 0;

let receivedData;
const outputElement = document.getElementById("output");

window.addEventListener('closeWindow', () => {
    console.log('Closing window...');
    const escapeEvent = new KeyboardEvent('keydown', {
        key: 'Escape',
        keyCode: 27,
        code: 'Escape',
        bubbles: true
    });
    window.dispatchEvent(escapeEvent);
});

const animateBackgroundSlide = () => {
    const ul = document.querySelector('ul');
    const listItems = document.querySelectorAll('li');
    
    listItems.forEach((li, index) => {
        if (index === actualIndex) {
            li.classList.add('active');
        } else {
            li.classList.remove('active');
        }
    });

    if (listItems[actualIndex] && ul) {
        const activeItem = listItems[actualIndex];
        const topOffset = activeItem.offsetTop;
        const height = activeItem.offsetHeight;
        
        ul.style.setProperty('--background-y', topOffset + 'px');
        ul.style.setProperty('--background-height', height + 'px');
        
        ul.classList.remove('animating');
        setTimeout(() => {
            ul.classList.add('animating');
        }, 10);
    }
};

const write = () => {
    let outputData = "<ul>";
    if (Array.isArray(receivedData)) {
        for (let i = 0; i < receivedData.length; i++) {
            outputData += `<li>${receivedData[i]}</li>`;
        }
        outputData += "</ul>";
        outputElement.innerHTML = outputData;
        
        setTimeout(() => {
            animateBackgroundSlide();
        }, 0);
        return;
    }
    else {
        outputElement.innerHTML = receivedData;
    }
}

const openApp = async () => {
    const dataToSend = {
        message: app,
    }

    await fetch("http://127.0.0.1:3000/run", {
        method: "POST",
        headers: {
            "Content-Type": "application/json",
        },
        body: JSON.stringify(dataToSend),
    });

    await new Promise(resolve => setTimeout(resolve, 300));
    
    document.title = 'CLOSE_WINDOW_SIGNAL';
}

inputField.addEventListener("keydown", (event) => {
    const key = event.key;

    switch (key) {
        case "ArrowUp":
            actualIndex = actualIndex > 0 ? actualIndex - 1 : 0;
            break;
        case "ArrowDown":
            actualIndex = actualIndex < maxIndex - 1 ? actualIndex + 1 : maxIndex - 1;
            break;
        case "Enter":
            openApp();
            break;
        default:
            return;
    }

    app = receivedData[actualIndex]

    animateBackgroundSlide();
});

inputField.addEventListener("input", async (event) => {
    try {
        const dataToSend = {
            message: event.target.value,
        };

        websocket.send(JSON.stringify(dataToSend));

        websocket.onmessage = (event) => {
            actualIndex = 0;
            receivedData = JSON.parse(event.data).message;
            console.log(receivedData)
            maxIndex = receivedData.length;
            write();

            app = receivedData[actualIndex]
        };

    } catch (error) {
        console.error("Error:", error);
    }
});