//! Membership with suspicion-based failure detection.

/// State of a member in the gossip cluster.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MemberState {
    Alive,
    Suspect,
    Dead,
}

/// A member in the cluster.
#[derive(Debug, Clone)]
pub struct Member {
    pub id: u64,
    pub state: MemberState,
    pub incarnation: u64,
    pub heartbeat: u64,
}

impl Member {
    pub fn new(id: u64) -> Self {
        Self { id, state: MemberState::Alive, incarnation: 0, heartbeat: 0 }
    }

    /// Record a heartbeat, clearing suspicion.
    pub fn ping(&mut self) {
        self.heartbeat += 1;
        if self.state == MemberState::Suspect {
            self.incarnation += 1;
            self.state = MemberState::Alive;
        }
    }

    /// Mark as suspect.
    pub fn suspect(&mut self) {
        if self.state == MemberState::Alive {
            self.state = MemberState::Suspect;
        }
    }

    /// Mark as dead.
    pub fn kill(&mut self) {
        self.state = MemberState::Dead;
    }
}

/// Membership list with suspicion mechanism.
#[derive(Debug, Clone)]
pub struct MembershipList {
    members: Vec<Member>,
    suspect_timeout: u64,
}

impl MembershipList {
    pub fn new(suspect_timeout: u64) -> Self {
        Self { members: Vec::new(), suspect_timeout }
    }

    /// Add a member.
    pub fn join(&mut self, id: u64) {
        if !self.members.iter().any(|m| m.id == id) {
            self.members.push(Member::new(id));
        }
    }

    /// Remove a member.
    pub fn leave(&mut self, id: u64) {
        self.members.retain(|m| m.id != id);
    }

    /// Record heartbeat from a member.
    pub fn heartbeat(&mut self, id: u64) {
        if let Some(m) = self.members.iter_mut().find(|m| m.id == id) {
            m.ping();
        }
    }

    /// Check for suspects: members that haven't heartbeated recently.
    pub fn check_suspects(&mut self, current_round: u64) {
        for m in &mut self.members {
            if m.state == MemberState::Alive && current_round > m.heartbeat + self.suspect_timeout {
                m.suspect();
            }
        }
    }

    /// Get alive members.
    pub fn alive_members(&self) -> Vec<&Member> {
        self.members.iter().filter(|m| m.state == MemberState::Alive).collect()
    }

    /// Get member by id.
    pub fn get(&self, id: u64) -> Option<&Member> {
        self.members.iter().find(|m| m.id == id)
    }

    /// Merge with another membership list (keep higher incarnation).
    pub fn merge(&mut self, other: &MembershipList) {
        for their_member in &other.members {
            if let Some(my_member) = self.members.iter_mut().find(|m| m.id == their_member.id) {
                if their_member.incarnation > my_member.incarnation
                    || (their_member.incarnation == my_member.incarnation && their_member.heartbeat > my_member.heartbeat)
                {
                    *my_member = their_member.clone();
                }
            } else {
                self.members.push(their_member.clone());
            }
        }
    }

    /// Number of members.
    pub fn len(&self) -> usize {
        self.members.len()
    }

    /// Is the list empty?
    pub fn is_empty(&self) -> bool {
        self.members.is_empty()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_join_and_leave() {
        let mut ml = MembershipList::new(3);
        ml.join(1);
        ml.join(2);
        assert_eq!(ml.len(), 2);
        ml.leave(1);
        assert_eq!(ml.len(), 1);
    }

    #[test]
    fn test_duplicate_join() {
        let mut ml = MembershipList::new(3);
        ml.join(1);
        ml.join(1);
        assert_eq!(ml.len(), 1);
    }

    #[test]
    fn test_heartbeat_updates() {
        let mut ml = MembershipList::new(3);
        ml.join(1);
        ml.heartbeat(1);
        assert_eq!(ml.get(1).unwrap().heartbeat, 1);
    }

    #[test]
    fn test_suspicion() {
        let mut ml = MembershipList::new(2);
        ml.join(1);
        ml.heartbeat(1); // heartbeat at round 1
        ml.check_suspects(5); // round 5, last hb=1, timeout=2 => suspect
        assert_eq!(ml.get(1).unwrap().state, MemberState::Suspect);
    }

    #[test]
    fn test_suspicion_clears_on_heartbeat() {
        let mut ml = MembershipList::new(2);
        ml.join(1);
        ml.heartbeat(1);
        ml.check_suspects(5);
        assert_eq!(ml.get(1).unwrap().state, MemberState::Suspect);
        ml.heartbeat(1); // clears suspicion, increments incarnation
        assert_eq!(ml.get(1).unwrap().state, MemberState::Alive);
        assert_eq!(ml.get(1).unwrap().incarnation, 1);
    }

    #[test]
    fn test_merge_keeps_higher_incarnation() {
        let mut ml1 = MembershipList::new(3);
        ml1.join(1);
        let mut ml2 = MembershipList::new(3);
        ml2.join(1);
        // Manually set higher incarnation on ml2's member
        for m in &mut ml2.members {
            if m.id == 1 {
                m.incarnation = 5;
                m.heartbeat = 10;
            }
        }
        ml1.merge(&ml2);
        assert_eq!(ml1.get(1).unwrap().incarnation, 5);
    }

    #[test]
    fn test_alive_members() {
        let mut ml = MembershipList::new(100);
        ml.join(1);
        ml.join(2);
        ml.join(3);
        // Heartbeat for 1 and 3 at round 50
        ml.heartbeat(1);
        ml.heartbeat(3);
        // Check suspects at round 51 — 1 and 3 should be alive, 2 has no heartbeat
        ml.check_suspects(51);
        let alive = ml.alive_members();
        assert!(alive.iter().any(|m| m.id == 1));
        assert!(alive.iter().any(|m| m.id == 3));
    }

    #[test]
    fn test_kill_member() {
        let mut m = Member::new(1);
        m.kill();
        assert_eq!(m.state, MemberState::Dead);
    }
}
