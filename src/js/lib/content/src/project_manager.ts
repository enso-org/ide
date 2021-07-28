/// This module defines Project Manager class.

const PROJECT_MANAGER_ENDPOINT = "ws://127.0.0.1:30535"

const MISSING_COMPONENT_ACTION_INSTALL = 'Install'

class ProjectManager {

    protected readonly connection_url: string

    constructor(connection_url: string) {
        this.connection_url = connection_url
    }

    static default() {
        return new ProjectManager(PROJECT_MANAGER_ENDPOINT)
    }

    listProjects() {
        const req =
        {
            jsonrpc: "2.0",
            id: 0,
            method: "project/list",
            params: {}
        }

        const ws = new WebSocket(this.connection_url)
        return new Promise((resolve, reject) => {
            ws.onopen = () => {
                console.log('onopen')
                ws.send(JSON.stringify(req))
            }
            ws.onmessage = (event: any) => {
                console.log('onmessage', event)
                resolve(JSON.parse(event.data))
            }
            ws.onerror = (error: any) => {
                console.log('onerror', error)
                reject(error)
            }
        }).finally(() => ws.close())
    }

    createProject(name: string, template?: string, action = MISSING_COMPONENT_ACTION_INSTALL) {
        let params = {
            name: name,
            missingComponentAction: action,
        }
        if (template !== undefined) {
            // @ts-ignore
            params["template"] = template
        }
        const req =
        {
            jsonrpc: "2.0",
            id: 0,
            method: "project/create",
            params: params
        }

        const ws = new WebSocket(this.connection_url)
        return new Promise((resolve, reject) => {
            ws.onopen = () => {
                console.log('onopen')
                ws.send(JSON.stringify(req))
            }
            ws.onmessage = (event) => {
                console.log('onmessage', event)
                resolve(JSON.parse(event.data))
            }
            ws.onerror = (error) => {
                console.log('onerror', error)
                reject(error)
            }
        }).finally(() => ws.close())
    }
}

export { ProjectManager }
