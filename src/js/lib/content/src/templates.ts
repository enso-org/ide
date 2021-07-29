/// This module defines helper methods for templates view.

import { ProjectManager } from './project_manager'

const PM = ProjectManager.default()

const CARD_SPREADSHEETS = 'card-spreadsheets'
const CARD_GEO = 'card-geo'
const CARD_VISUALIZE = 'card-visualize'
const CARD_BMW_DRIVERS = 'card-bmw-drivers'

const PROJECTS_LIST = 'projects-list'
const PROJECTS_LIST_NEW_PROJECT = 'projects-list-new-project'

const ALL_CARDS = [
    CARD_SPREADSHEETS,
    CARD_GEO,
    CARD_VISUALIZE,
    CARD_BMW_DRIVERS
]

/// Sore for hidden elements.
///
/// When the templates view is loaded, we hide some top-level elements, as their
/// style is messing up scrolling. Hidden elements will be restored before
/// loading the IDE.
let hiddenElements: HTMLDivElement[] = []

/// Status box div element for displaying errors.
let statusBox: HTMLElement = undefined

async function loadTemplatesView(openProject: (project: string) => void) {
    const templatesView = require('./templates-view.html')
    hideRootHtml()
    document.body.innerHTML += templatesView
    statusBox = document.getElementById('templates-status-box')
    console.log("templates loaded")

    try {
        await loadProjectsList(openProject)
    } catch (error) {
        displayStatusBox("Failed to load projects.")
    }
    console.log("projects loaded")

    setTemplateCardHandlers(openProject)
    console.log("template handlers set")
}

function hideRootHtml() {
    const matches = document.body.querySelectorAll('div')
    matches.forEach(element => {
        hiddenElements.push(element)
        element.remove()
    })
}

function restoreRootHtml() {
    let templatesView = document.getElementById('templates-view')
    hiddenElements
        .slice()
        .reverse()
        .forEach(element => document.body.prepend(element))
    hiddenElements = []
    templatesView.remove()
}

function displayStatusBox(text: string): void {
    statusBox.innerHTML = text
    statusBox.style.visibility = 'visible'
}

function clearStatusBox(): void {
    statusBox.style.visibility = 'hidden'
}
async function loadProjectsList(openProject: (project: string) => void) {
    const projectsListNode = document.getElementById(PROJECTS_LIST)

    const newProjectNode = document.getElementById(PROJECTS_LIST_NEW_PROJECT)
    newProjectNode.setAttribute('style', 'cursor: pointer;')
    newProjectNode.onclick = () => {
        console.log('newProjectNode.onclick')
        clearStatusBox()
        PM.createProject('Unnamed', 'default')
            .then((response: any) => {
                console.log('createProject', response)
                if (response.error !== undefined) {
                    console.error(response.error.message)
                    displayStatusBox(response.error.message)
                } else {
                    restoreRootHtml()
                    openProject(response.result.projectName)
                }
            })
            .catch((error: any) => {
                console.error('newProjectNode.onclick', error)
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

function buildProjectListNode(projectName: string, openProject: (project: string) => void) {
    const li = document.createElement('li')
    li.setAttribute('style', 'cursor: pointer;')
    li.onclick = () => {
        console.log('li.onclick ' + projectName)
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

function setTemplateCardHandlers(openProject: (project: String) => void) {
    ALL_CARDS.forEach((cardId: string) => {
        const cardElement = document.getElementById(cardId)
        setTemplateCardHandler(cardElement, openProject)
    })
}

function setTemplateCardHandler(element: HTMLElement, openProject: (project: string) => void) {
    element.setAttribute('style', 'cursor: pointer;')
    element.onclick = () => {
        const projectName = getProjectName(element.id)
        const templateName = getProjectTemplate(element.id)
        clearStatusBox()
        console.log('onclick ', element.id, projectName, templateName)

        PM.createProject(projectName, templateName)
            .then((response: any) => {
                console.log('createProject', response)
                if (response.error !== undefined) {
                    console.error(response.error.message)
                    displayStatusBox(response.error.message)
                } else {
                    restoreRootHtml()
                    openProject(response.result.projectName)
                }
            })
            .catch((error: any) => {
                console.error('template.onclick', error)
                displayStatusBox("Failed to open a template.")
            })

    }
}

function getProjectName(elementId: string): string {
    switch (elementId) {
        case CARD_SPREADSHEETS:
            return 'Spreadsheets'
        case CARD_GEO:
            return 'Geo'
        case CARD_VISUALIZE:
            return 'Visualize'
        case CARD_BMW_DRIVERS:
            return 'Bmw_Drivers'
        default:
            return 'Template'
    }
}

function getProjectTemplate(elementId: string): string {
    switch (elementId) {
        case CARD_SPREADSHEETS:
            return 'example'
        case CARD_GEO:
            return 'example'
        case CARD_VISUALIZE:
            return 'example'
        case CARD_BMW_DRIVERS:
            return 'example'
        default:
            return 'default'
    }
}

export { loadTemplatesView }
