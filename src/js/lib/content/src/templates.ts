/// This module defines helper methods for templates view.

import { ProjectManager } from './project_manager'

const PM = ProjectManager.default()

let hiddenElements: any[] = []

function loadTemplatesView() {
    const templatesView = require('./templates-view.html')
    hideRootHtml()
    document.body.innerHTML += templatesView
    //restoreRootHtml()
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
    templatesView.remove()
}

async function loadProjectsList(openProject: (project: string) => any) {
    const projectsListNode = document.getElementById('projects-list')
    const newProjectNode = document.getElementById('projects-list-new-project')
    newProjectNode.onclick = () => {
        console.log('newProjectNode.onclick')
        PM.createProject('Unnamed', 'default')
            .then((response: any) => {
                console.log('createProject', response)
                if (response.error !== undefined) {
                    console.error(response.error.message)
                } else {
                    restoreRootHtml()
                    openProject(response.result.projectName)
                }
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

function buildProjectListNode(projectName: string, openProject: (project: string) => any) {
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

export { loadTemplatesView, loadProjectsList }
