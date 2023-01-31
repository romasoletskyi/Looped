import { ClientDatabase, ClientChat } from "wasm-core"

const serverURL = "https://104.155.14.233:3000";

const jobs = [
    "Farmer",
    "Fisherman",
    "Miner",
    "Merchant",
    "Politician",
    "Noble",
    "Priest"
]

const traits = [
    "rebellion",
    "fear_propension",
    "popularity",
    "animosity",
    "political_agreement",
    "fear"
]

const minTrait = -10;
const maxTrait = 10;

// returns number from Uniform[start, end]
function randomInteger(start, end) {
    return Math.floor(Math.random() * (end + 1 - start)) + start;
}

const approach = document.getElementById("poll-job");
const jobDescription = document.getElementById("job-selector");
const traitList = document.getElementById("poll-character");
const phraseList = document.getElementById("poll-phrase");
const chatButton = document.getElementById("start");
const generateButton = document.getElementById("generate");
const chatHistory = document.getElementById("chat-history");
const saveButton = document.getElementById("save-button");
const dataSize = document.getElementById("data-size");
const mode = document.getElementById("mode");

function initialize() {
    for (const job of jobs) {
        const op = document.createElement("option");
        op.text = job;
        jobDescription.add(op);
    }

    chatButton.addEventListener("click", () => {
        if (chatButton.textContent == "Chat") {
            chatHistory.textContent = "";
            chatButton.textContent = "Restart";
            generateButton.style.visibility = "hidden";

            chat = ClientChat.new(database, youTalk, serializePerson());
            chat.start();

            postPhrases();
        } else {
            chatHistory.textContent = "";
            generateButton.style.visibility = "visible";

            restartSituation();
        }
    });

    generateButton.addEventListener("click", () => {
        chatHistory.textContent = "";
        chatButton.textContent = "Chat";
        phraseList.textContent = "";

        const oldyouTalk = youTalk;
        chat = ClientChat.new(database, youTalk, serializePerson());

        let phrases = chat.get_phrases();
        while (phrases.length > 0) {
            const index = randomInteger(0, phrases.length - 1);
            const phrase = phrases[index];

            chat.choose_phrase_immutably(index);
            updateChatHistory(phrase, 0);

            phrases = chat.get_phrases();
        }

        youTalk = oldyouTalk;
    });

    saveButton.addEventListener("click", saveConversationBackup);
}

function restartSituation() {
    youTalk = randomInteger(0, 1) === 0;

    if (youTalk) {
        approach.textContent = "You approach a ";
    } else {
        approach.textContent = "You are approached by a";
    }

    jobDescription.selectedIndex = randomInteger(0, jobDescription.length - 1);

    traitList.textContent = "";
    for (const trait of traits) {
        const op = document.createElement("div");

        const slider = document.createElement("input");
        slider.type = "range";
        slider.id = "slider-" + trait;
        slider.min = minTrait.toString();
        slider.max = maxTrait.toString();
        slider.value = randomInteger(minTrait, maxTrait).toString();

        const label = document.createElement("label");
        label.textContent = trait;
        label.htmlFor = slider.id;

        op.appendChild(label);
        op.appendChild(slider);
        traitList.appendChild(op);
    }

    if (chat) {
        updateDatabase();
    } else {
        chatButton.textContent = "Chat";
    }

    if (online) {
        mode.textContent = "online";
    } else {
        mode.textContent = "offline";
    }
}

function serializePerson() {
    let character = {}
    for (const trait of traits) {
        character[trait] = parseInt(document.getElementById("slider-" + trait).value);
    }
    return JSON.stringify({"job": jobs[jobDescription.selectedIndex], "character": character});
}

let youTalk = randomInteger(0, 1) === 0;
let databaseSize = 0;
let chat = null;
let online = false;

const database = ClientDatabase.new();

function loadDatabase() {
    let xmlHttp = new XMLHttpRequest();
    xmlHttp.onreadystatechange = () => { 
        if (xmlHttp.readyState == 4) {
            if (xmlHttp.status == 200) {
                const server_database = ClientDatabase.from_str(xmlHttp.responseText);
                if (server_database) {
                    database.merge(server_database);
                }
                online = true;
            }
            databaseSize = database.size();
            dataSize.textContent = databaseSize.toString();
            initialize();
            restartSituation();
        }
    };
    xmlHttp.open("GET", serverURL + "/database", true);
    xmlHttp.send();
}

loadDatabase();

function updateDatabase() {
    let xmlHttp = new XMLHttpRequest();
    xmlHttp.onreadystatechange = () => { 
        if (xmlHttp.readyState == 4) {
            if (xmlHttp.status == 200) {
                const difference = ClientDatabase.from_str(xmlHttp.responseText);
                if (difference) {
                    database.merge(difference);
                }
                online = true;
            } else {
                online = false;
            }
            chatButton.textContent = "Chat";
        }
    };
    xmlHttp.open("POST", serverURL + "/database", true);
    xmlHttp.send(database.difference().to_string());
}

function postPhrases() {
    const phrases = chat.get_phrases();
    phraseList.textContent = "";

    for (const [index, phrase] of phrases.entries()) {
        const op = document.createElement("div");
        op.classList.add("variant");
        op.setAttribute("onclick", "location.href='#';");
        op.style = "cursor: pointer;";

        op.textContent = phrase;
        op.addEventListener("click", () => {
            chat.choose_phrase(index);
            updateChatHistory(phrase, 1);
            postPhrases();
        });

        phraseList.appendChild(op);
    }

    const op = document.createElement("div");
    op.classList.add("variant");

    const add = document.createElement("div");
    add.id = "add-answer";
    add.textContent = "Add answer";
    add.setAttribute("onclick", "location.href='#';");
    add.style = "cursor: pointer;";
    op.appendChild(add);
    
    add.addEventListener("click", () => {
        const line = document.createElement("input");
        line.type = "input";
        line.id = "poll-answer";
        
        line.addEventListener("keydown", (event) => {
            if (event.key == "Enter") {
                chat.add_phrase(line.value);
                updateChatHistory(line.value, 1);
                postPhrases();
            }
        });

        op.appendChild(line);
    });

    phraseList.appendChild(op);
}

function updateChatHistory(message, delta) {
    const op = document.createElement("div");
    op.textContent = message;

    if (youTalk) {
        op.classList.add("message");
        op.classList.add("your");
    } else {
        op.classList.add("message");
    }

    chatHistory.appendChild(op);

    databaseSize += delta
    dataSize.textContent = databaseSize.toString();
    youTalk = !youTalk;
}

function saveConversationBackup() {
    const a = document.createElement("a");
    a.href = window.URL.createObjectURL(new Blob([database.to_string()], {type: "text/plain"}));
    a.download = "database.json";
    a.click();
}
