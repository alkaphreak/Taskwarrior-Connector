browser.tabs.query({active: true, currentWindow: true})
    .then(tabs => {
        for (const tab of tabs) {
            document.getElementById("title").value = tab.title;
            document.getElementById("url").value = tab.url;
        }
});

// state: "pending" | "success" | "info" | "error" | "offline" (styled in popup.html)
function setStatus(text, state) {
    const status = document.getElementById("status");
    status.textContent = text;
    status.dataset.state = state;
}

document.addEventListener('DOMContentLoaded', function () {
    // Title and URL come pre-filled from the tab, so tags is the usual next input.
    document.getElementById("tags").focus();
    const submitBtn = document.getElementById("submit");
    submitBtn.addEventListener('click', function (evt) {
        const titleValue = document.getElementById("title").value.trim();
        const urlValue = document.getElementById("url").value.trim();
        if (!titleValue || !urlValue) {
            setStatus("Title and URL are required.", "error");
            return;
        }
        const body = new URLSearchParams({
            title: titleValue,
            url: urlValue,
            tag: document.getElementById("tags").value
        });
        submitBtn.disabled = true;
        setStatus("Saving…", "pending");
        fetch("http://127.0.0.1:34810/", {
            method: "POST",
            headers: {"Content-Type": "application/x-www-form-urlencoded"},
            body: body.toString()
        })
        .then(r => r.json())
        .then(j => {
            if (!j.ok) {
                setStatus("Error: " + j.error, "error");
                return;
            }
            setStatus(j.duplicate ? "Already saved." : "Saved.", j.duplicate ? "info" : "success");
            // Clear the form on any successful outcome (saved or already-saved) —
            // nothing left for the user to act on, keeps a stray click from
            // re-submitting the same values.
            document.getElementById("title").value = "";
            document.getElementById("url").value = "";
            document.getElementById("tags").value = "";
        })
        .catch(e => {
            setStatus("Error: connector not running (" + e + ")", "offline");
        })
        .finally(() => {
            submitBtn.disabled = false;
        });
    });
});
