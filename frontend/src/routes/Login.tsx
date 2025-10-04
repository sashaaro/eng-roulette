import Login from "./../component/Login.tsx";

export default function LoginPage() {
    // const session = useContext(SessionContext);

    return (
        <div className="form-signin">
            <div className="h3 mb-3 font-weight-normal">
                Please sign in
            </div>
            <Login register_mode={false}/>
        </div>
    );
}
