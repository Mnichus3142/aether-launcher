const inputField = document.getElementById("userInput");

const wsUri = "ws://127.0.0.1:3000";
const websocket = new WebSocket(wsUri);

let actualIndex = 0;
let maxIndex = 0;
let app = "";
let searchInWeb = false
let previousIndex = 0;
let lastRenderedData = null;
let isDOMChanged = false;

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

const animateBackgroundSlide = (animate = true) => {
    const ul = document.querySelector('ul');
    const listItems = document.querySelectorAll('li');
    
    listItems.forEach((li, index) => {
        if (index === actualIndex) {
            li.classList.add('active');
        } else {
            li.classList.remove('active');
        }
    });

    if (animate && previousIndex !== actualIndex && !isDOMChanged && listItems[actualIndex] && ul) {
        const activeItem = listItems[actualIndex];
        const topOffset = activeItem.offsetTop;
        const height = activeItem.offsetHeight;
        
        ul.style.setProperty('--background-y', topOffset + 'px');
        ul.style.setProperty('--background-height', height + 'px');
        
        ul.classList.remove('animating');
        setTimeout(() => {
            ul.classList.add('animating');
        }, 10);
        
        previousIndex = actualIndex;
    } else {
        if (listItems[actualIndex] && ul) {
            const activeItem = listItems[actualIndex];
            const topOffset = activeItem.offsetTop;
            const height = activeItem.offsetHeight;
            
            ul.style.setProperty('--background-y', topOffset + 'px');
            ul.style.setProperty('--background-height', height + 'px');
            
            previousIndex = actualIndex;
        }
    }
};

const write = () => {
    console.log(inputField.value);
    const actualData = inputField.value;
    let outputData = "<ul>";
    if (Array.isArray(receivedData)) {
        for (let i = 0; i < receivedData.length; i++) {
            outputData += `<li>${receivedData[i]}</li>`;
        }
        outputData += `<li>Look for ${actualData} in web</li>`;
        outputData += "</ul>";
        
        lastRenderedData = JSON.stringify(receivedData) + actualData;
        outputElement.innerHTML = outputData;
        
        isDOMChanged = true;
        
        const ul = document.querySelector('ul');
        if (ul) {
            ul.classList.add('no-transition');
        }
        
        animateBackgroundSlide(false);
        
        setTimeout(() => {
            if (ul) {
                ul.classList.remove('no-transition');
            }
            isDOMChanged = false;
        }, 0);

        return;
    }
    else {
        outputElement.innerHTML = `<p class="equasionResult">${receivedData}</p>`;
    }
}

const openApp = async () => {
    const dataToSend = {
        message: app,
        searchInWeb: searchInWeb
    };

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
            event.preventDefault();
            actualIndex--;
            actualIndex <= 0 ? actualIndex = 0 : actualIndex;
            break;
        case "ArrowDown":
            event.preventDefault();
            actualIndex++;
            actualIndex >= maxIndex + 1 ? actualIndex = maxIndex : actualIndex;
            break;
        case "Enter":
            if (actualIndex === maxIndex) {
                searchInWeb = true;
                app = inputField.value;
                console.log(app)
            }
            openApp();
            break;
        default:
            return;

        }

    app = receivedData[actualIndex]

    animateBackgroundSlide(true);
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
            maxIndex = receivedData.length;
            write();

            app = receivedData[actualIndex]
        };

    } catch (error) {
        console.error("Error:", error);
    }
});