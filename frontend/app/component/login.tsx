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
        let tokenResponse = register_mode ?
            await accountService.register(data.username, data.password) :
            await accountService.login(data.username, data.password)

        setUser({username: data.username, id: 1})

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
                <button type="submit">Login</button>
            </div>
        </form>
    );
}