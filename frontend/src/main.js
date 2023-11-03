import $ from "jquery";
import {env} from "./env/env";

let clientsSet = new Set();

function isClientListDifferent(clients) {
    if (clients.length != clientsSet.size) {
        return true;
    }
    for (let client of clients) {
        if (!clientsSet.has(client.id)) {
            return true;
        }
    }
    return false;
}

function updateClientSet(clients) {
    clientsSet.clear();
    for (let client of clients) {
        clientsSet.add(client.id);
    }
}

function pullClients(force = false) {
    $.ajax({
        url: `${env.api}/clients`,
        data: {},
        success: function (result) {
            const clients = JSON.parse(result);

            if (force || isClientListDifferent(clients)) {
                console.log('is diff')
                updateClientSet(clients);
                $('#clientList').empty();
                for (let i = 0; i < clients.length; i++) {
                    $('#clientList').append(`
<tr>
    <td class="border-b border-slate-100 dark:border-slate-700 p-4 pl-8 text-slate-500 dark:text-slate-400">
        ${clients[i].name}
    </td>
    <td class="border-b border-slate-100 dark:border-slate-700 p-4 pl-8 text-slate-500 dark:text-slate-400">
        ${clients[i].key}
    </td>
    <td class="border-b border-slate-100 dark:border-slate-700 p-4 pl-8 text-slate-500 dark:text-slate-400">
        <button onclick="window.deleteClient(${clients[i].id})" class="rounded-full hover:bg-red-300 bg-red-500 w-20 text-sm leading-5 font-semibold text-white">
            delete
        </button>
        <button onclick="window.genClientKey(${clients[i].id})" class="rounded-full hover:bg-lime-300 bg-lime-500 w-20 text-sm leading-5 font-semibold text-white">
            genKey
        </button>
<!--        <button class="rounded-full hover:bg-lime-300 bg-lime-500 w-20 text-sm leading-5 font-semibold text-white">edit</button>-->
    </td>
</tr>`);
                }
            }
        }
    });
}

function deleteClient(id) {
    $.ajax({
        url: `${env.api}/clients/${id}`,
        type: 'DELETE',
        success: function (result) {
            console.log('Removed', result);
            pullClients(true);
        }
    });
}

window.deleteClient = deleteClient;

function newClient() {
    $.ajax({
        url: `${env.api}/clients`,
        type: 'POST',
        datatype: 'json',
        data: {name: $('#newClientNameEle').val()},
        success: function (result) {
            console.log('New', result);
            pullClients(true);
        }
    });
}

window.newClient = newClient;

function genClientKey(id) {
    $.ajax({
        url: `${env.api}/clients/${id}/generate_key`,
        type: 'POST',
        datatype: 'json',
        data: {name: $('#newClientNameEle').val()},
        success: function (result) {
            console.log('Generated key', result);
            pullClients(true);
        }
    });
}

window.genClientKey = genClientKey;

function closeCreateClientDialog() {
    $('#createNewClientDialog').prop('open', false);
}

window.closeCreateClientDialog = closeCreateClientDialog;

function openCreateClientDialog() {
    $('#newClientNameEle').val("");
    $('#createNewClientDialog').prop('open', true);
}

window.openCreateClientDialog = openCreateClientDialog;

pullClients();
setInterval(pullClients, env.pullingInterval);
