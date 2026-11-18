const list = document.getElementById("projects");
const status = document.getElementById("status");

loadProjects().then(projects => { list.value = projects.join("\n"); });

document.getElementById("save").addEventListener("click", async () => {
    const projects = [...new Set(list.value.split("\n").map(p => p.trim()).filter(Boolean))];
    const bad = projects.filter(p => !PROJECT_RE.test(p));
    list.setAttribute("aria-invalid", String(bad.length > 0));
    if (bad.length) {
        status.textContent = "Invalid: " + bad.join(", ") + " (letters, digits, . - _ only)";
        status.dataset.state = "error";
        return;
    }
    await browser.storage.sync.set({projects});
    list.value = (projects.length ? projects : DEFAULT_PROJECTS).join("\n");
    status.textContent = "Saved. Default project: " + (projects[0] || DEFAULT_PROJECTS[0]);
    status.dataset.state = "success";
});
