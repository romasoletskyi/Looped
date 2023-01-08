import { Database, Chat } from "wasm-core"

const jobs = [
    "farmer",
    "merchant",
    "priest"
]

const traits = [
    "hostile",
    "rebellious"
]

const minTrait = -5;
const maxTrait = 5;

// returns number from Uniform[start, end]
function randomInteger(start, end) {
    return Math.floor(Math.random() * (end + 1 - start)) + start
}

const approach = document.getElementById("poll-job");
const jobDescription = document.getElementById("job-selector");
const traitList = document.getElementById("poll-character");
const phraseList = document.getElementById("poll-phrase");
const chatButton = document.getElementById("start");
const chatHistory = document.getElementById("chat-history");

function initialize() {
    for (const job of jobs) {
        const op = document.createElement("option");
        op.text = job;
        jobDescription.add(op);
    }

    chatButton.addEventListener("click", () => {
        if (chatButton.textContent == "Chat") {
            chatButton.textContent = "Restart";
            postPhrases();
        } else {
            chatButton.textContent = "Chat";
            chat = Chat.new(database, youTalk);
            restartSituation();
        }
    });
}

function restartSituation() {
    youTalk = randomInteger(0, 1) === 0;

    if (youTalk) {
        approach.textContent = "You approach a ";
    } else {
        approach.textContent = "You are approached by a";
    }

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

    chatHistory.textContent = "";
}

let youTalk = randomInteger(0, 1) === 0;
initialize();
restartSituation();

const database = Database.new();
let chat = Chat.new(database, youTalk);

function postPhrases() {
    const phrases = chat.get_phrases();
    console.log(phrases);
    phraseList.textContent = "";

    for (const [index, phrase] of phrases.entries()) {
        const op = document.createElement("div");
        op.classList.add("variant");
        op.setAttribute("onclick", "location.href='#';");
        op.style = "cursor: pointer;";

        op.textContent = phrase;
        op.addEventListener("click", () => {
            chat.choose_phrase(index);
            updateChatHistory(phrase);
            youTalk = !youTalk;
            postPhrases();
        });

        phraseList.appendChild(op);
    }

    const op = document.createElement("div");
    op.classList.add("variant");

    const add = document.createElement("div");
    add.id = "add-answer";
    add.textContent = "Add answer";
    add.setAttribute("onclick", "location.href='#';")
    add.style = "cursor: pointer;";
    op.appendChild(add);
    
    add.addEventListener("click", () => {
        const line = document.createElement("input");
        line.type = "input";
        line.id = "poll-answer";
        
        line.addEventListener("keydown", (event) => {
            if (event.key == "Enter") {
                chat.add_phrase(line.value);
                updateChatHistory(line.value);
                youTalk = !youTalk;
                postPhrases();
            }
        });

        op.appendChild(line);
    });

    phraseList.appendChild(op);
}

function updateChatHistory(message) {
    const op = document.createElement("div");
    op.textContent = message;

    if (youTalk) {
        op.classList.add("message");
        op.classList.add("your");
    } else {
        op.classList.add("message");
    }

    chatHistory.appendChild(op);
}

function saveConversationBackup() {
    const blob = new Blob([])
}
