import {accountService} from "./../service/account.ts";
import {SessionContext, useAuth} from "./../context/session.tsx";
import { useForm } from "react-hook-form"
import {useNavigate} from "react-router";
import {useContext} from "react";

type Inputs = {
    username: string
    password: string
}

export default function Login({register_mode} : {register_mode: boolean}){
    const session = useContext(SessionContext);

    const {
        register,
        handleSubmit,
        watch,
        formState: { errors },
    } = useForm<Inputs>()

    const {user, setUser} = useAuth();
    let navigate = useNavigate();

    const onSubmit = async (data: Inputs) => {
        const tokenResponse = register_mode ?
            await accountService.register(data.username, data.password) :
            await accountService.login(data.username, data.password)

        const token = tokenResponse.token;
        const user = await accountService.me(token)

        setUser({username: user.username, id: user.id, token: token})

        navigate("/")
        // save jwt session
        //session
    }

    return (
        <form className="login-form" onSubmit={handleSubmit(onSubmit)}>
            <input className="form-control" type="text" placeholder={"username"} {...register("username")}/>
            <input className="form-control" type="password" placeholder={"password"} {...register("password")}/>
            <button className="btn btn-lg btn-primary btn-block" type="submit">{register_mode ? 'Create account' : 'Sign in'}</button>
        </form>
    );
}