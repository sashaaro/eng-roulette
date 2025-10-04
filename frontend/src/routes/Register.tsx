import Login from "./../component/Login.tsx";

export default function Register() {
    return (
        <div className="form-signin">
            <div className="h3 mb-3 font-weight-normal">
                Please register an account
            </div>
            <Login register_mode={true}/>
        </div>
    );
}
