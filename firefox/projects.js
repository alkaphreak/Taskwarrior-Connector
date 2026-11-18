// Project preferences shared by the popup and the options page.
// Same rule as the connector's valid_project(): letters, digits, ".", "-", "_", 64 max.
const PROJECT_RE = /^[A-Za-z0-9._-]{1,64}$/;
const DEFAULT_PROJECTS = ["Links"];

// The first project is the default one.
async function loadProjects() {
    try {
        const {projects} = await browser.storage.sync.get({projects: DEFAULT_PROJECTS});
        return Array.isArray(projects) && projects.length ? projects : DEFAULT_PROJECTS;
    } catch (e) {
        return DEFAULT_PROJECTS;
    }
}
