import {useAuth} from "./../context/session.tsx";
import {Link} from "react-router";
import {useCallback} from "react";
import {JoinRoom} from "./../component/JoinRoom.tsx";
import {accountService} from "./../service/account.ts";

export default function Home() {
    let {user, loading, setUser} = useAuth();

    const onExit = useCallback((e) => {
        e.preventDefault();
        setUser(null);
        localStorage.removeItem("session"); // TODO refactoring
        // TODO roomService.ws.close();
    }, [])

    const googleLogin = useCallback(async () => {
        const resp = await accountService.googleAuth(window.location.origin)
        localStorage.setItem("pkce_code_verifier", resp.pkce_code_verifier)
        open(resp.authorize_url)
    }, []);

    return (
        <div>
            {loading ? <div>Loading...</div> : null}
            {!loading ? <ul className="nav flex-column">
                {!user ? <li className="nav-item"><Link className="nav-link" to={"/login"}>Login</Link></li> : null}
                {!user ? <li className="nav-item"><a href="#" className="nav-link" onClick={googleLogin}>Google Login</a></li> : null}
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
