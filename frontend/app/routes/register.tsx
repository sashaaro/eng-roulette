import type { Route } from "./+types/home";
import {SessionContext} from "~/context/session";
import {useContext} from "react";
import Login from "~/component/Login";

export default function RegisterPage({ loaderData }: Route.ComponentProps) {
    const session = useContext(SessionContext);


    return (
        <div className="form-signin">
            <div className="h3 mb-3 font-weight-normal">
                Please register an account
            </div>
            <Login register_mode={true}/>
        </div>
    );
}
