/// This module defines helper methods for templates view.

import { ProjectManager } from './project_manager'

const PM = ProjectManager.default()

const PROJECTS_LIST = 'projects-list'
const PROJECTS_LIST_NEW_PROJECT = 'projects-list-new-project'

const CARD_SPREADSHEETS = 'card-spreadsheets'
const CARD_GEO = 'card-geo'
const CARD_VISUALIZE = 'card-visualize'

const ALL_CARDS = [
    CARD_SPREADSHEETS,
    CARD_GEO,
    CARD_VISUALIZE,
]

const VID_INTRO = 'vid-intro'
const VID_ENSO2 = 'vid-enso2'
const VID_JAVA = 'vid-java'
const VID_COMPILER = 'vid-compiler'

const VID_INTRO_URL = 'https://youtu.be/wFkh5LgAZTs'
const VID_ENSO2_URL = 'https://youtu.be/rF8DuJPOfTs'
const VID_JAVA_URL = 'https://youtu.be/bcpOEX1x06I'
const VID_COMPILER_URL = 'https://youtu.be/BibjcUjdkO4'

/**
 * The sore for hidden elements.
 *
 * When the templates view is loaded, it hides some top-level elements, as
 * their style is messing up scrolling. Hidden elements will be restored before
 * loading the IDE.
 */
let hiddenElements: HTMLDivElement[] = []

/** Status box div element for displaying errors. */
let statusBox: HTMLElement = undefined

/**
 * Display the templates view.
 *
 * Main entry point. Loads the templates HTML markup, loads the projects list
 * and sets callbacks on the template cards.
 *
 * @param openProject the callback that opens IDE with the provided project.
 */
async function loadTemplatesView(openProject: (project: string) => void): Promise<void> {
    const templatesView = require('./templates-view.html')
    hideRootHtml()
    document.body.innerHTML += templatesView
    statusBox = document.getElementById('templates-status-box')

    try {
        await loadProjectsList(openProject)
    } catch (error) {
        displayStatusBox("Failed to load projects.")
    }

    setTemplateCardHandlers(openProject)
    setYoutubeTutorialHandlers()
}

/**
 * Remove the top-level div elements from the scene.
 */
function hideRootHtml(): void {
    const matches = document.body.querySelectorAll('div')
    matches.forEach(element => {
        hiddenElements.push(element)
        element.remove()
    })
}

/**
 * Restore the elements removed by the `hideRootHtml` function.
 */
function restoreRootHtml(): void {
    let templatesView = document.getElementById('templates-view')
    hiddenElements
        .slice()
        .reverse()
        .forEach(element => document.body.prepend(element))
    hiddenElements = []
    templatesView.remove()
}

/**
 * Show the message in the statsus box div element.
 *
 * @param text the message to display
 */
function displayStatusBox(text: string): void {
    statusBox.innerHTML = text
    statusBox.style.visibility = 'visible'
}

/**
 * Clear the status box div element.
 */
function clearStatusBox(): void {
    statusBox.style.visibility = 'hidden'
}

/**
 * Load the projects list.
 *
 * Uses Project Manager to get the list of user projects and displays
 * them in the projects side menu.
 *
 * @param openProject the callback that opens IDE with the provided project
 */
async function loadProjectsList(openProject: (project: string) => void): Promise<void> {
    const projectsListNode = document.getElementById(PROJECTS_LIST)

    const newProjectNode = document.getElementById(PROJECTS_LIST_NEW_PROJECT)
    newProjectNode.setAttribute('style', 'cursor: pointer;')
    newProjectNode.onclick = () => {
        clearStatusBox()
        PM.createProject('Unnamed', 'default')
            .then((response: any) => {
                if (response.error !== undefined) {
                    console.error('Project manager openProject failed', response)
                    displayStatusBox(response.error.message)
                } else {
                    restoreRootHtml()
                    openProject(response.result.projectName)
                }
            })
            .catch((error: any) => {
                console.error('onclick', PROJECTS_LIST_NEW_PROJECT, error)
                displayStatusBox("Failed to create a new project.")
            })
    }

    const projectsListResult = await PM.listProjects()
    const projectsList = projectsListResult
        .result
        .projects
        .map((project: any) => buildProjectListNode(project.name, openProject))

    projectsList.forEach((element: any) => {
        projectsListNode.insertBefore(element, newProjectNode)
    })
}

/**
 * Build `li` HTML element for the projects side menu.
 *
 * @param projectName the name of the project
 * @param openProject the callback that opens IDE with the provided project
 */
function buildProjectListNode(projectName: string, openProject: (project: string) => void): HTMLLIElement {
    const li = document.createElement('li')
    li.setAttribute('style', 'cursor: pointer;')
    li.onclick = () => {
        restoreRootHtml()
        openProject(projectName)
    }

    const img = document.createElement('img')
    img.setAttribute('src', '/assets/project.svg')

    const text = document.createTextNode(projectName)

    li.appendChild(img)
    li.appendChild(text)

    return li
}

/**
 * Set `onclick` callbacks for all template cards.
 *
 * @param openProject the callback that opens IDE with the provided project
 */
function setTemplateCardHandlers(openProject: (project: String) => void): void {
    ALL_CARDS.forEach((cardId: string) => {
        const cardElement = document.getElementById(cardId)
        setTemplateCardHandler(cardElement, openProject)
    })
}

/**
 * Set the `onclick` callback for the template card.
 *
 * @param element the HTML element of the template card
 * @param openProject the callback that opens IDE with the provided project
 */
function setTemplateCardHandler(element: HTMLElement, openProject: (project: string) => void): void {
    element.setAttribute('style', 'cursor: pointer;')
    element.onclick = () => {
        const projectName = getProjectName(element.id)
        const templateName = getProjectTemplate(element.id)
        clearStatusBox()

        PM.createProject(projectName, templateName)
            .then((response: any) => {
                if (response.error !== undefined) {
                    console.error("Project manager createProject failed", response)
                    displayStatusBox(response.error.message)
                } else {
                    restoreRootHtml()
                    openProject(response.result.projectName)
                }
            })
            .catch((error: any) => {
                console.error('onclick', element.id, error)
                displayStatusBox("Failed to open a template.")
            })
    }
}

/**
 * Get the project name by the template card HTML identifier.
 *
 * @param elementId the template card id
 * @return the project name
 */
function getProjectName(elementId: string): string {
    switch (elementId) {
        case CARD_SPREADSHEETS:
            return 'Orders'
        case CARD_GEO:
            return 'Restaurants'
        case CARD_VISUALIZE:
            return 'Stargazers'
        default:
            return 'Unnamed'
    }
}

/**
 * Get the template name by the template card HTML identifier.
 *
 * @param elementId the template card id
 * @return the template name
 */
function getProjectTemplate(elementId: string): string {
    switch (elementId) {
        case CARD_SPREADSHEETS:
            return 'orders'
        case CARD_GEO:
            return 'restaurants'
        case CARD_VISUALIZE:
            return 'stargazers'
        default:
            return 'default'
    }
}

/**
 * Set the `onclick` callback for all the tutorial videos.
 */
function setYoutubeTutorialHandlers(): void {
    setYoutubeTutorialHandler(VID_INTRO, VID_INTRO_URL)
    setYoutubeTutorialHandler(VID_ENSO2, VID_ENSO2_URL)
    setYoutubeTutorialHandler(VID_JAVA, VID_JAVA_URL)
    setYoutubeTutorialHandler(VID_COMPILER, VID_COMPILER_URL)
}

/**
 * Set the `onclick` callback for the tutorial video.
 *
 * @param elementId the HTML id of the tutorial card
 * @param link the link to the YouTube video
 */
function setYoutubeTutorialHandler(elementId: string, link: string): void {
    const element = document.getElementById(elementId)
    element.setAttribute('style', 'cursor: pointer;')
    element.onclick = () => {
        window.open(link, 'newwindow', 'width=1280,height=720')
        return false
    }
}


export { loadTemplatesView }
