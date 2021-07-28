/// This module defines helper methods for templates view.

import { ProjectManager } from './project_manager'

const PM = ProjectManager.default()

function loadTemplatesView() {
    const templatesView = require('./templates-view.html')
    const root = document.getElementById('root')
    root.insertAdjacentHTML('beforeend', templatesView)
}

async function loadProjectsList() {
    const projectsListNode = document.getElementById('projects-list')
    const newProjectNode = document.getElementById('projects-list-new-project')

    const projectsListResult = await PM.listProjects()
    const projectsList = buildProjectsList(projectsListResult)

    projectsList.forEach((element: any) => {
        projectsListNode.insertBefore(element, newProjectNode)
    })
}

function buildProjectsList(projectsList: any) {
    const projectNodes = projectsList
        .result
        .projects
        .map((project: any) => toProjectListNode(project.name))

    return projectNodes
}

function toProjectListNode(projectName: string) {
    const li = document.createElement('li')
    li.innerHTML = projectName

    return li
}

export { loadTemplatesView, loadProjectsList }
