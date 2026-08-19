browser.tabs.query({active: true, currentWindow: true})
    .then(tabs => {
        for (const tab of tabs) {
            document.getElementById("title").value = tab.title;
            document.getElementById("url").value = tab.url;
        }
});

document.addEventListener('DOMContentLoaded', function () {
    const submitBtn = document.getElementById("submit");
    submitBtn.addEventListener('click', function (evt) {
        const titleValue = document.getElementById("title").value.trim();
        const urlValue = document.getElementById("url").value.trim();
        if (!titleValue || !urlValue) {
            document.getElementById("status").textContent = "Title and URL are required.";
            return;
        }
        const body = new URLSearchParams({
            title: titleValue,
            url: urlValue,
            tag: document.getElementById("tags").value
        });
        submitBtn.disabled = true;
        document.getElementById("status").textContent = "Saving…";
        fetch("http://127.0.0.1:34810/", {
            method: "POST",
            headers: {"Content-Type": "application/x-www-form-urlencoded"},
            body: body.toString()
        })
        .then(r => r.json())
        .then(j => {
            if (!j.ok) {
                document.getElementById("status").textContent = "Error: " + j.error;
                return;
            }
            document.getElementById("status").textContent = j.duplicate ? "Already saved." : "Saved.";
            // Clear the form on any successful outcome (saved or already-saved) —
            // nothing left for the user to act on, keeps a stray click from
            // re-submitting the same values.
            document.getElementById("title").value = "";
            document.getElementById("url").value = "";
            document.getElementById("tags").value = "";
        })
        .catch(e => {
            document.getElementById("status").textContent = "Error: connector not running (" + e + ")";
        })
        .finally(() => {
            submitBtn.disabled = false;
        });
    });
});
