import $ from "jquery";
import {env} from "./env/env";

$.ajax({
    url: `${env.api}/clients`,
    data: {},
    success: function( result ) {
        console.log(result);
    }
});
