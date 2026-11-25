browser.tabs.query({active: true, currentWindow: true})
    .then(tabs => {
        for (const tab of tabs) {
            document.getElementById("title").value = tab.title;
            document.getElementById("url").value = tab.url;
        }
});

// state: "idle" | "pending" | "success" | "info" | "error" | "offline" (styled in popup.html)
function setStatus(text, state) {
    const status = document.getElementById("status");
    status.textContent = text;
    status.dataset.state = state;
}

// Red border on an empty required field (MarsTech-Skin .input[aria-invalid])
function markInvalid(id, invalid) {
    document.getElementById(id).setAttribute("aria-invalid", String(invalid));
}

// Status line at rest shows where the task will go.
function showProject() {
    setStatus("project:" + document.getElementById("project").value + " · POST localhost only", "idle");
}

document.addEventListener('DOMContentLoaded', function () {
    const select = document.getElementById("project");
    const submitBtn = document.getElementById("submit");
    const fields = ["title", "url", "tags", "description"].map(id => document.getElementById(id));
    // No save before the project list is in: an empty select would file it under Links.
    submitBtn.disabled = true;
    loadProjects().then(projects => {
        for (const p of projects) select.add(new Option(p, p));
        showProject();
        submitBtn.disabled = false;
    });
    select.addEventListener("change", showProject);
    // Title and URL come pre-filled from the tab, so tags is the usual next input.
    document.getElementById("tags").focus();
    for (const id of ["title", "url"]) {
        document.getElementById(id).addEventListener("input", () => markInvalid(id, false));
    }
    submitBtn.addEventListener('click', function (evt) {
        const titleValue = document.getElementById("title").value.trim();
        const urlValue = document.getElementById("url").value.trim();
        markInvalid("title", !titleValue);
        markInvalid("url", !urlValue);
        if (!titleValue || !urlValue) {
            setStatus("Title and URL are required.", "error");
            return;
        }
        const body = new URLSearchParams({
            title: titleValue,
            url: urlValue,
            tag: document.getElementById("tags").value,
            project: select.value,
            description: document.getElementById("description").value
        });
        submitBtn.disabled = true;
        // Read-only while pending: a success clears the form, edits made meanwhile would be lost.
        for (const f of fields) f.readOnly = true;
        select.disabled = true;
        submitBtn.textContent = "Saving…";
        setStatus("Saving…", "pending");
        let closing = false;
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
            const description = document.getElementById("description").value.trim();
            if (j.duplicate) {
                // Nothing was written: keep the form so a typed description is not lost.
                setStatus(description ? "Already saved, description not added." : "Already saved.", "info");
                return;
            }
            if (j.warning) {
                // Task saved, annotation lost: keep the description so it can be added by hand.
                setStatus("Saved, but " + j.warning, "info");
                return;
            }
            if (j.project === undefined && (select.value !== "Links" || description)) {
                // A connector from before MARSTECH-912 ignores project and description.
                setStatus("Saved to project:Links only: connector too old, update it.", "info");
                return;
            }
            setStatus("Saved to project:" + (j.project || "Links") + ".", "success");
            // Clear the form once saved —
            // nothing left for the user to act on, keeps a stray click from
            // re-submitting the same values.
            document.getElementById("title").value = "";
            document.getElementById("url").value = "";
            document.getElementById("tags").value = "";
            document.getElementById("description").value = "";
            // Nothing left to act on: close, form kept locked so input typed meanwhile is not lost.
            closing = true;
            submitBtn.textContent = "Saved";
            setTimeout(() => window.close(), 1500);
        })
        .catch(e => {
            setStatus("Error: connector not running (" + e + ")", "offline");
        })
        .finally(() => {
            if (closing) return;
            submitBtn.disabled = false;
            for (const f of fields) f.readOnly = false;
            select.disabled = false;
            submitBtn.textContent = "Add to Taskwarrior";
        });
    });
});
