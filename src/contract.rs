use cosmwasm_std::{
    to_binary, Api, Binary, Env, Extern, HandleResponse, InitResponse, Querier, StdError,
    StdResult, Storage,
};

use crate::msg::{HandleMsg, InitMsg, QueryMsg};
use crate::state::{config, config_read, State};

pub fn init<S: Storage, A: Api, Q: Querier>(
    deps: &mut Extern<S, A, Q>,
    env: Env,
    msg: InitMsg,
) -> StdResult<InitResponse> {
    let state = State {
        count: msg.count,
        owner: env.message.sender,
    };

    config(&mut deps.storage).save(&state)?;

    Ok(InitResponse::default())
}

pub fn handle<S: Storage, A: Api, Q: Querier>(
    deps: &mut Extern<S, A, Q>,
    env: Env,
    msg: HandleMsg,
) -> StdResult<HandleResponse> {
    match msg {
        HandleMsg::Increase { value } => try_increase(deps, env, value),
        HandleMsg::Decrease { value } => try_decrease(deps, env, value),
        HandleMsg::Reset { count } => try_reset(deps, env, count),
    }
}

pub fn try_increase<S: Storage, A: Api, Q: Querier>(
    deps: &mut Extern<S, A, Q>,
    _env: Env,
    value: i32,
) -> StdResult<HandleResponse> {
    config(&mut deps.storage).update(|mut state| {
        state.count += value;
        Ok(state)
    })?;

    Ok(HandleResponse::default())
}

pub fn try_decrease<S: Storage, A: Api, Q: Querier>(
    deps: &mut Extern<S, A, Q>,
    _env: Env,
    value: i32,
) -> StdResult<HandleResponse> {
    config(&mut deps.storage).update(|mut state| {
        state.count -= value;
        Ok(state)
    })?;

    Ok(HandleResponse::default())
}


pub fn try_reset<S: Storage, A: Api, Q: Querier>(
    deps: &mut Extern<S, A, Q>,
    env: Env,
    count: i32,
) -> StdResult<HandleResponse> {
    config(&mut deps.storage).update(|mut state| {
        if env.message.sender != state.owner {
            return Err(StdError::Unauthorized { backtrace: None });
        }
        state.count = count;
        Ok(state)
    })?;
    Ok(HandleResponse::default())
}

pub fn query<S: Storage, A: Api, Q: Querier>(
    deps: &Extern<S, A, Q>,
    msg: QueryMsg,
) -> StdResult<Binary> {
    match msg {
        QueryMsg::QueryEvenOdd {} => to_binary(&query_even_odd(deps)?),
    }
}

fn query_even_odd<S: Storage, A: Api, Q: Querier>(deps: &Extern<S, A, Q>) -> StdResult<String> {
    let state = config_read(&deps.storage).load()?;
    if state.count % 2 == 0 {
        Ok(format!("Even Number: {}", state.count))
    } else {
        Ok(format!("Odd Number: {}", state.count))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use cosmwasm_std::testing::{mock_dependencies, mock_env};
    use cosmwasm_std::{coins, from_binary, StdError};

    #[test]
    fn proper_initialization() {
        let mut deps = mock_dependencies(20, &[]);

        let msg = InitMsg { count: 17 };
        let env = mock_env(&deps.api, "creator", &coins(1000, "earth"));

        // we can just call .unwrap() to assert this was a success
        let res = init(&mut deps, env, msg).unwrap();
        assert_eq!(0, res.messages.len());

        // it worked, let's query the state
        let res = query(&deps, QueryMsg::QueryEvenOdd {}).unwrap();
        let value: String = from_binary(&res).unwrap();
        assert_eq!("Odd Number: 17", value);
    }

    #[test]
    fn increase() {
        let mut deps = mock_dependencies(20, &coins(2, "token"));

        let msg = InitMsg { count: 17 };
        let env = mock_env(&deps.api, "creator", &coins(2, "token"));
        let _res = init(&mut deps, env, msg).unwrap();

        // beneficiary can release it
        let env = mock_env(&deps.api, "anyone", &coins(2, "token"));
        let msg = HandleMsg::Increase { value: 2 };
        let _res = handle(&mut deps, env, msg).unwrap();

        let res = query(&deps, QueryMsg::QueryEvenOdd {}).unwrap();
        let value: String = from_binary(&res).unwrap();
        assert_eq!("Odd Number: 19", value);
    }

    #[test]
    fn decrease() {
        let mut deps = mock_dependencies(20, &coins(2, "token"));

        let msg = InitMsg { count: 17 };
        let env = mock_env(&deps.api, "creator", &coins(2, "token"));
        let _res = init(&mut deps, env, msg).unwrap();

        // beneficiary can release it
        let env = mock_env(&deps.api, "anyone", &coins(2, "token"));
        let msg = HandleMsg::Decrease { value: 1 };
        let _res = handle(&mut deps, env, msg).unwrap();

        let res = query(&deps, QueryMsg::QueryEvenOdd {}).unwrap();
        let value: String = from_binary(&res).unwrap();
        assert_eq!("Even Number: 16", value);
    }

    #[test]
    fn reset() {
        let mut deps = mock_dependencies(20, &coins(2, "token"));

        let msg = InitMsg { count: 17 };
        let env = mock_env(&deps.api, "creator", &coins(2, "token"));
        let _res = init(&mut deps, env, msg).unwrap();

        // beneficiary can release it
        let unauth_env = mock_env(&deps.api, "anyone", &coins(2, "token"));
        let msg = HandleMsg::Reset { count: 5 };
        let res = handle(&mut deps, unauth_env, msg);
        match res {
            Err(StdError::Unauthorized { .. }) => {}
            _ => panic!("Must return unauthorized error"),
        }

        // only the original creator can reset the counter
        let auth_env = mock_env(&deps.api, "creator", &coins(2, "token"));
        let msg = HandleMsg::Reset { count: 5 };
        let _res = handle(&mut deps, auth_env, msg).unwrap();

        // should now be 5
        let res = query(&deps, QueryMsg::QueryEvenOdd {}).unwrap();
        let value: String = from_binary(&res).unwrap();
        assert_eq!("Odd Number: 5", value);
    }
}
