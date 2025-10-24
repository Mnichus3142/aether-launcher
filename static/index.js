const inputField = document.getElementById("userInput");

const wsUri = "ws://127.0.0.1:3000";
const websocket = new WebSocket(wsUri);

inputField.addEventListener("input", async (event) => {
    try {
        const dataToSend = {
            message: event.target.value,
        };

        websocket.send(JSON.stringify(dataToSend));

        websocket.onmessage = (event) => {
            const receivedData = JSON.parse(event.data);
            console.log("Received from server:", JSON.parse(event.data).message);
            document.getElementById("output").innerHTML = `${receivedData.message}`;
        };

    } catch (error) {
        console.error("Error:", error);
    }
});