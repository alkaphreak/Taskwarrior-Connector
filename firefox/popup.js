browser.tabs.query({active: true, currentWindow: true})
    .then(tabs => {
        for (const tab of tabs) {
            document.getElementById("title").value = tab.title;
            document.getElementById("url").value = tab.url;
        }
});

document.addEventListener('DOMContentLoaded', function () {
    document.getElementById("submit").addEventListener('click', function (evt) {
        const body = new URLSearchParams({
            title: document.getElementById("title").value,
            url: document.getElementById("url").value,
            tag: document.getElementById("tags").value
        });
        fetch("http://127.0.0.1:34810/", {
            method: "POST",
            headers: {"Content-Type": "application/x-www-form-urlencoded"},
            body: body.toString()
        })
        .then(r => r.json())
        .then(j => {
            if (!j.ok) {
                document.getElementById("status").textContent = "Error: " + j.error;
            } else {
                document.getElementById("status").textContent = j.duplicate ? "Already saved." : "Saved.";
            }
        })
        .catch(e => {
            document.getElementById("status").textContent = "Error: connector not running (" + e + ")";
        });
    });
});
