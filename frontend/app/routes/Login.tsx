import type { Route } from "./+types/home";
import {SessionContext} from "~/context/session";
import {useContext} from "react";
import Login from "~/component/Login";

export function loader() {
  return { name: "React Router" };
}

export default function LoginPage({ loaderData }: Route.ComponentProps) {
    const session = useContext(SessionContext);


    return (
        <div className="form-signin">
            <div className="h3 mb-3 font-weight-normal">
                Please sign in
            </div>
            <Login register_mode={false}/>
        </div>
    );
}
