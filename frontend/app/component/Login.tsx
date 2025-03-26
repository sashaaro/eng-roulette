import {type FunctionComponent, useCallback, useContext} from "react";
import {accountService} from "~/service/account";
import {SessionContext, useAuth} from "~/context/session";
import { useForm } from "react-hook-form"
import {useNavigate} from "react-router";

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
        <form onSubmit={handleSubmit(onSubmit)}>
            <div>
                <input type="text" {...register("username")}/>
            </div>
            <div>
                <input type="password" {...register("password")}/>
            </div>
            <div>
                <button type="submit">{register_mode ? 'Register' : 'Login'}</button>
            </div>
        </form>
    );
}