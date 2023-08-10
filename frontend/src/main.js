import $ from "jquery";
import {env} from "./env/env";

function pullClients() {
    $.ajax({
        url: `${env.api}/clients`,
        data: {},
        success: function (result) {
            const clients = JSON.parse(result);
            // console.log(clients);
            $('#clientList').empty();
            for (let i = 0; i < clients.length; i++) {
                $('#clientList').append(`<tr><td>${clients[i].name}</td></tr>`);
            }
        }
    });
}

setInterval(pullClients, 1000);
