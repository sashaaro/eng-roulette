import type { Route } from "./+types/Home";
import {useAuth} from "~/context/session";
import {Link, useLocation} from "react-router";
import {useCallback, useEffect, useState} from "react";
import {JoinRoom} from "~/component/JoinRoom";
import {accountService} from "~/service/account";

export default function Home({ loaderData }: Route.ComponentProps) {
    let {user, loading, setUser} = useAuth();

    const onExit = useCallback((e) => {
        e.preventDefault();
        setUser(null);
        localStorage.removeItem("session"); // TODO refactoring
        // TODO roomService.ws.close();
    }, [])


    const googleLogin = useCallback(async () => {
        const link = await accountService.googleAuth()
        open(link)
    }, []);

    return (
        <div>
            {loading ? <div>Loading...</div> : null}
            {!loading ? <ul className="nav flex-column">
                {!user ? <li className="nav-item"><Link className="nav-link" to={"/login"}>Login</Link></li> : null}
                {!user ? <li className="nav-item"><Link className="nav-link" onClick={googleLogin}>Google Login</Link></li> : null}
                {!user ? <li className="nav-item"><Link className="nav-link" to={"/register"}>Register</Link></li> : null}
                {user ? <li className="nav-item"><a className="nav-link" href="#" onClick={onExit}>Exit</a></li>: null}
            </ul> :null}
            {!loading ? <div>
                {user ? <div>
                    <div className="text-center h3">Hi {user.username}</div>
                    <JoinRoom/>
                </div> : null}
            </div> :null}
        </div>
    );
}
