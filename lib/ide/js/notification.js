class NotificationService {
    constructor() {
        this.dom = document.createElement("div");
        var style = `
            display: flex;
            flex-direction: column;
            position: absolute;
            top: 0px;
            left: 0px;
            width: 100vw;
            height: 100vh;
            justify-content: flex-end;
            align-items: flex-end;
        `.replace(/\n| /g, "");
        this.dom.setAttribute("style", style);
        document.body.appendChild(this.dom);
    }
}

class Notification {
    constructor(service,content,duration,fade_out) {
        var duration   = duration   || 1.0;
        var fade_out = fade_out || 1.0;
        this.dom       = document.createElement("div");
        var style      = `
            font-family: "Arial";
            background-color: rgb(32, 32, 32);
            color: white;
            width: 320px;
            position: relative;
            bottom: 0px;
            border: 1px solid black;
            border-radius: 3px 3px;
            margin-right: 10px;
            margin-bottom: 10px;
            padding: 5px;
            transition: all ${fade_out}s ease-in-out;
        `;
        this.dom.setAttribute("style", style);
        this.dom.innerHTML = `<center><b>${content}</b></center>`;
        service.dom.prepend(this.dom);
        var notification = this;
        this.timer = setTimeout(function() {
            notification.dom.style.opacity = "0";
            notification.fader_timer       = setTimeout(function() {
                notification.dom.parentNode.removeChild(notification.dom);
            }, fade_out * 1000);
        }, duration * 1000);
    }
}

export function create_notification_service() {
    return new NotificationService();
}

export function create_notification(service,content,duration,fade_out) {
    new Notification(service,content,duration,fade_out)
}
