use crate::prelude::*;

use futures::channel::oneshot;

use crate::common::error::NoSuchRequest;


#[derive(Debug,Default)]
pub struct OngoingCalls<Id,Reply> where Id:Hash+Eq {
    logger        : Logger,
    ongoing_calls : HashMap<Id,oneshot::Sender<Reply>>,
}

impl<Id,Reply> OngoingCalls<Id,Reply>
where Id:Copy + Debug + Display + Hash + Eq + Send + Sync + 'static {
    pub fn new(parent_logger:&Logger) -> OngoingCalls<Id,Reply> {
        OngoingCalls {
            logger : parent_logger.sub("ongoing_calls"),
            ongoing_calls : HashMap::new(),
        }
    }

    pub fn remove_request(&mut self, id:&Id) -> Option<oneshot::Sender<Reply>> {
        let ret = self.ongoing_calls.remove(id);
        if ret.is_some() {
            info!(self.logger,"Removing request {id}");
        } else {
            info!(self.logger,"Failed to remove non-present request {id}");
        }
        ret
    }

    pub fn insert_request(&mut self, id:Id, sender:oneshot::Sender<Reply>) {
        info!(self.logger,"Storing a new request {id}");
        // There will be no previous request, since Ids are assumed to be unique.
        // Still, if there was, we can just safely drop it.
        let _ = self.ongoing_calls.insert(id,sender);
    }

    pub fn open_new_request<F,R>(&mut self, id:Id, f:F) -> impl Future<Output=FallibleResult<R>>
    where F: FnOnce(Reply) -> FallibleResult<R> {
        let (sender, receiver) = oneshot::channel::<Reply>();
        let logger = self.logger.clone_ref();
        let ret = receiver.map(move |result_or_cancel| {
            let result = result_or_cancel?;
            f(result)
        });
        self.insert_request(id, sender);
        ret
    }

    pub fn clear(&mut self) {
        info!(self.logger,"Clearing all the requests.");
        self.ongoing_calls.clear()
    }

    pub fn complete_request(&mut self, id:Id, reply:Reply) -> FallibleResult<()> {
        if let Some(mut request) = self.remove_request(&id) {
            // Explicitly ignore error. Can happen only if the other side already dropped future
            // with the call result. In such case no one needs to be notified and we are fine.
            let _ = request.send(reply);
            Ok(())
        } else {
            Err(NoSuchRequest(id).into())
        }
    }
}
