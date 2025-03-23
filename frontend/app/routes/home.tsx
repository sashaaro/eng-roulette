import type { Route } from "./+types/home";
import {SessionContext, useAuth} from "~/context/session";
import {useContext} from "react";
import Login from "~/component/login";
import {Link} from "react-router";

export function loader() {
  return { name: "React Router" };
}

export default function Home({ loaderData }: Route.ComponentProps) {
    const session = useContext(SessionContext);
    const {user} = useAuth();


    return (
        <div className="text-center p-4">
            <h1 className="text-3xl font-bold underline">
                Home
                {user ? <div>Hi {user.username}</div> : null}
            </h1>
            <nav>
                <Link to={"/login"}>Login</Link>
                <Link to={"/register"}>Register</Link>
            </nav>
        </div>
    );
}
