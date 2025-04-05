import type { Route } from "./+types/home";
import {SessionContext} from "~/context/session";
import {useContext} from "react";
import Login from "~/component/Login";

export function loader() {
  return { name: "React Router" };
}

export default function RegisterPage({ loaderData }: Route.ComponentProps) {
    const session = useContext(SessionContext);


    return (
        <div className="text-center p-4">
            <h1 className="text-3xl font-bold underline">
                Register new user
            </h1>
            <Login register_mode={true}/>
        </div>
    );
}
