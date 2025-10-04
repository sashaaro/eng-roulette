import {useAuth} from "./../context/session.tsx";
import {useEffect} from "react";
import {accountService} from "./../service/account.ts";
import {useNavigate, useSearchParams} from "react-router";

export default function GoogleAuthCallback() {
    const [searchParams, _] = useSearchParams();
    const {user, setUser} = useAuth();
    let navigate = useNavigate();


    useEffect(()=> {
        (async () => {
            const token = await accountService.googleAuthCallback(
                searchParams.get("state")!,
                searchParams.get("code")!,
                localStorage.getItem("pkce_code_verifier")!,
                window.location.origin
            )

            const user = await accountService.me(token)

            setUser({username: user.username, id: user.id, token: token})

            navigate("/")
        })()
    });

    return (
        <div className="form-signin">
            <div className="h3 mb-3 font-weight-normal">
                Loading....
            </div>
        </div>
    );
}